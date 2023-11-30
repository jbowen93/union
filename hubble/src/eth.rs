use std::{ops::Range, time::Duration};

use color_eyre::Report;
use ethers::{
    providers::{Http, Middleware, Provider},
    types::{BlockNumber, H256},
};
use futures::{
    stream::{self, FuturesOrdered},
    TryStreamExt,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Postgres};
use time::OffsetDateTime;
use tracing::{debug, info};
use url::Url;

use crate::{
    metrics,
    postgres::{self, ChainId},
};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Config {
    pub url: Url,
}

pub struct Indexer {
    range: Range<u64>,
    chain_id: postgres::ChainId,
    tasks: tokio::task::JoinSet<Result<(), Report>>,
    pool: PgPool,
    provider: Provider<Http>,
}

impl Config {
    pub async fn indexer(self, pool: PgPool) -> Result<Indexer, Report> {
        let provider = Provider::<Http>::try_from(self.url.clone().as_str()).unwrap();

        info!("fetching chain-id from node");
        let chain_id = provider.get_chainid().await?.as_u64();

        let chain_id = postgres::fetch_or_insert_chain_id(&pool, chain_id.to_string())
            .await?
            .get_inner_logged();

        let current = sqlx::query!(
            r#"SELECT height 
            FROM "v0"."logs" 
            WHERE chain_id = $1 
            ORDER BY height DESC 
            NULLS LAST 
            LIMIT 1"#,
            chain_id.db
        )
        .fetch_optional(&pool)
        .await?
        .map(|block| {
            if block.height == 0 {
                info!("no block found, starting at 0");
                0
            } else {
                info!("block found, continueing at {}", block.height + 1);
                block.height + 1
            }
        })
        .unwrap_or_default() as u64;

        let range = current..u64::MAX;

        Ok(Indexer {
            range,
            tasks: tokio::task::JoinSet::new(),
            chain_id,
            pool,
            provider,
        })
    }
}
impl Indexer {
    /// Spawns two long running tasks, one which continuously indexes from current to head, and one which indexes from current-20 to current.
    /// The second routine handles fixing up block reorgs.
    pub async fn index(mut self) -> Result<(), Report> {
        info!(
            self.chain_id.cannonical,
            "starting indexing from {} to {}", self.range.start, self.range.end
        );

        debug!(self.chain_id.cannonical, "spawning main indexing routine");
        self.tasks.spawn(index_blocks(
            self.pool.clone(),
            self.range,
            self.chain_id,
            self.provider.clone(),
        ));

        debug!(self.chain_id.cannonical, "spawning fork indexing routine");
        self.tasks.spawn(reindex_blocks(
            self.pool.clone(),
            self.chain_id,
            self.provider.clone(),
        ));

        self.tasks
            .join_next()
            .await
            .expect("set has at least one task")??;
        Ok(())
    }
}

async fn index_blocks(
    pool: PgPool,
    range: Range<u64>,
    chain_id: ChainId,
    provider: Provider<Http>,
) -> Result<(), Report> {
    futures::stream::iter(range.into_iter().map(Ok::<u64, Report>))
        .try_chunks(200)
        .map_err(|err| err.1)
        .try_fold(pool, |pool, chunk| async {
            let tx = pool.begin().await.map_err(Report::from)?;
            let inserts = FuturesOrdered::from_iter(
                chunk
                    .into_iter()
                    .map(|height| BlockInsert::from_provider_retried(chain_id, height, &provider)),
            );

            let tx = inserts
                .try_fold(tx, |mut tx, (_, block)| async move {
                    match block.execute(&mut tx).await {
                        Err(err) => {
                            tx.rollback().await?;
                            return Err(err);
                        }
                        Ok(info) => {
                            info!(
                                chain_id.cannonical,
                                height = info.height,
                                hash = info.hash,
                                num_transactions = info.num_tx,
                                num_events = info.num_events,
                                "indexed block"
                            );
                            metrics::BLOCK_COLLECTOR
                                .with_label_values(&[chain_id.cannonical])
                                .inc();
                            metrics::TRANSACTION_COLLECTOR
                                .with_label_values(&[chain_id.cannonical])
                                .inc_by(info.num_tx as u64);
                            metrics::EVENT_COLLECTOR
                                .with_label_values(&[chain_id.cannonical])
                                .inc_by(info.num_events as u64);
                        }
                    }
                    Ok(tx)
                })
                .await?;
            tx.commit().await?;
            Ok(pool)
        })
        .await?;
    Ok(())
}

/// A worker routine which continuously re-indexes the last 20 blocks into `postgres::Logs`.
async fn reindex_blocks(
    pool: PgPool,
    chain_id: ChainId,
    provider: Provider<Http>,
) -> Result<(), Report> {
    loop {
        // Set start to the current height so we can continue re-indexing.
        let current = fetch_latest_height(&pool, chain_id).await?;
        let latest = provider
            .get_block(BlockNumber::Latest)
            .await?
            .expect("provider should have latest block");
        if latest.number.unwrap().as_u32() - current as u32 > 32 {
            tokio::time::sleep(Duration::from_secs(12 * 32)).await;
            continue;
        }

        let tx = pool.begin().await?;
        let chunk = (current - 20)..current;
        let inserts =
            FuturesOrdered::from_iter(chunk.into_iter().map(|height| {
                BlockInsert::from_provider_retried(chain_id, height as u64, &provider)
            }));
        inserts
            .try_fold(tx, |mut tx, (_, block)| async move {
                let log = postgres::Log {
                    chain_id: block.chain_id,
                    block_hash: block.hash.clone(),
                    height: block.height,
                    time: block.time,
                    data: LogData {
                        header: block.header,
                        transactions: block.transactions,
                    },
                };
                postgres::upsert_log(&mut tx, log).await?;
                Ok(tx)
            })
            .await?;
    }
}

async fn fetch_latest_height(pool: &PgPool, chain_id: ChainId) -> Result<i32, sqlx::Error> {
    let latest = sqlx::query!(
        "SELECT height FROM \"v0\".logs 
        WHERE chain_id = $1 
        ORDER BY height DESC 
        NULLS LAST 
        LIMIT 1",
        chain_id.db
    )
    .fetch_optional(pool)
    .await?
    .map(|block| block.height)
    .unwrap_or_default();
    Ok(latest)
}

pub struct InsertInfo {
    height: i32,
    hash: String,
    num_tx: usize,
    num_events: i32,
}

pub struct BlockInsert {
    chain_id: ChainId,
    hash: String,
    header: ethers::types::Block<H256>,
    height: i32,
    time: OffsetDateTime,
    transactions: Vec<TransactionInsert>,
}

#[derive(Serialize, Deserialize)]
pub struct LogData {
    pub transactions: Vec<TransactionInsert>,
    pub header: ethers::types::Block<H256>,
}

impl BlockInsert {
    async fn from_provider_retried(
        chain_id: ChainId,
        height: u64,
        provider: &Provider<Http>,
    ) -> Result<(usize, Self), Report> {
        let count = 0;
        let max_retries = 100;
        loop {
            match Self::from_provider(chain_id, height, provider).await {
                Ok(block) => return Ok((count, block)),
                Err(err) => {
                    if count > max_retries {
                        return Err(err);
                    }
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            }
        }
    }

    async fn from_provider(
        chain_id: ChainId,
        height: u64,
        provider: &Provider<Http>,
    ) -> Result<Self, Report> {
        let block = provider.get_block(height).await?.unwrap();
        let mut receipts = provider.get_block_receipts(height).await.unwrap();
        let ts = if block.number.unwrap().is_zero() {
            match chain_id.cannonical {
                "1" => 1438269973,
                "11155111" => 1691344421,
                id => todo!(
                    "chain-id {:?} unsupported. only ethereum and sepolia genesis are supported",
                    id
                ),
            }
        } else {
            block.time()?.timestamp()
        };
        let time = OffsetDateTime::from_unix_timestamp(ts)?;
        let block_hash = block.hash.unwrap().to_lower_hex();
        let height: i32 = block.number.unwrap().as_u32().try_into()?;
        receipts.sort_by(|a, b| a.transaction_index.cmp(&b.transaction_index));

        let transactions = receipts
            .into_iter()
            .map(|receipt| {
                let transaction_hash = receipt.transaction_hash.to_lower_hex();
                let transaction_index = receipt.transaction_index.as_u32() as i32;

                let events = receipt
                    .clone()
                    .logs
                    .into_iter()
                    .enumerate()
                    .map(|(transaction_log_index, log)| {
                        let data = serde_json::to_value(&log).unwrap();
                        EventInsert {
                            data,
                            log_index: log.log_index.unwrap().as_usize(),
                            transaction_log_index: transaction_log_index as i32,
                        }
                    })
                    .collect();
                TransactionInsert {
                    hash: transaction_hash,
                    data: receipt,
                    index: transaction_index,
                    events,
                }
            })
            .collect();

        Ok(BlockInsert {
            chain_id,
            hash: block_hash,
            header: block,
            height,
            time,
            transactions,
        })
    }

    /// Handles inserting the block data and transactions as a log.
    async fn execute(self, tx: &mut sqlx::Transaction<'_, Postgres>) -> Result<InsertInfo, Report> {
        let num_tx = self.transactions.len();
        let num_events = self
            .transactions
            .iter()
            .map(|tx| tx.events.len() as i32)
            .sum();

        let log = postgres::Log {
            chain_id: self.chain_id,
            block_hash: self.hash.clone(),
            height: self.height,
            time: self.time,
            data: LogData {
                header: self.header,
                transactions: self.transactions,
            },
        };

        postgres::insert_batch_logs(tx, stream::iter(Some(log))).await?;

        Ok(InsertInfo {
            height: self.height,
            hash: self.hash,
            num_tx,
            num_events,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TransactionInsert {
    hash: String,
    data: ethers::types::TransactionReceipt,
    index: i32,
    events: Vec<EventInsert>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EventInsert {
    data: serde_json::Value,
    log_index: usize,
    transaction_log_index: i32,
}

pub trait ToLowerHex {
    fn to_lower_hex(&self) -> String;
}

impl ToLowerHex for H256 {
    fn to_lower_hex(&self) -> String {
        format!("{:#x}", self)
    }
}
