use std::{net::SocketAddr, str::FromStr};

use clap::Parser;
use tracing::{info_span, Instrument};
use url::Url;

use crate::logging::LogFormat;

/// Hubble is state machine observer.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The url to the hasura graphql endpoint.
    #[arg(short, long, env = "HUBBLE_HASURA_URL")]
    pub url: Option<Url>,

    /// The database url used to connect with timescaledb.
    #[arg(
        group = "datastore",
        required = true,
        short,
        long,
        env = "HUBBLE_DATABASE_URL"
    )]
    pub database_url: Option<String>,

    /// Indexer configurations to start.
    #[arg(short, long, env = "HUBBLE_INDEXERS")]
    pub indexers: Indexers,

    /// Indexer configurations to start.
    #[arg(short, long, env = "HUBBLE_METRICS_PORT")]
    pub metrics_addr: Option<SocketAddr>,

    /// Fetch the counterparty chain ids for all clients known to hubble.
    #[arg(long)]
    pub fetch_client_chain_ids: bool,

    /// The log format for Hubble.
    #[arg(
        global = true,
        short = 'f',
        long,
        env = "HUBBLE_LOG_FORMAT",
        default_value = "json"
    )]
    pub log_format: LogFormat,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Indexers(Vec<IndexerConfig>);

impl IntoIterator for Indexers {
    type Item = IndexerConfig;

    type IntoIter = std::vec::IntoIter<IndexerConfig>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(tag = "type")]
#[allow(clippy::large_enum_variant)]
pub enum IndexerConfig {
    #[serde(rename = "tendermint")]
    Tm(crate::tm::Config),
    #[serde(rename = "ethereum")]
    Eth(crate::eth::Config),
    #[serde(rename = "ethereum-fork")]
    EthFork(crate::eth::fork::Config),
    #[serde(rename = "beacon")]
    Beacon(crate::beacon::Config),
    #[serde(rename = "bera")]
    Bera(crate::bera::Config),
    #[serde(rename = "arb")]
    Arb(crate::arb::Config),
    #[serde(rename = "scroll")]
    Scroll(crate::scroll::Config),
}

impl IndexerConfig {
    pub fn label(&self) -> &str {
        match &self {
            Self::Tm(cfg) => &cfg.label,
            Self::Eth(cfg) => &cfg.label,
            Self::Beacon(cfg) => &cfg.label,
            Self::Bera(cfg) => &cfg.label,
            Self::EthFork(cfg) => &cfg.label,
            Self::Arb(cfg) => &cfg.label,
            Self::Scroll(cfg) => &cfg.label,
        }
    }
}

impl IndexerConfig {
    pub async fn index(self, db: sqlx::PgPool) -> Result<(), color_eyre::eyre::Report> {
        let label = self.label();

        let initializer_span = info_span!("initializer", label);
        let indexer_span = info_span!("indexer", label);

        match self {
            Self::Tm(cfg) => cfg.index(db).instrument(indexer_span).await,
            Self::Eth(cfg) => {
                cfg.indexer(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::Beacon(cfg) => {
                cfg.indexer(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::Bera(cfg) => {
                cfg.indexer(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::EthFork(cfg) => {
                cfg.indexer(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::Arb(cfg) => {
                cfg.indexer(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
            Self::Scroll(cfg) => {
                cfg.indexer(db)
                    .instrument(initializer_span)
                    .await?
                    .index()
                    .instrument(indexer_span)
                    .await
            }
        }
    }
}

impl FromStr for Indexers {
    type Err = color_eyre::eyre::Error;

    fn from_str(item: &str) -> Result<Self, <Self as FromStr>::Err> {
        serde_json::from_str(item).map_err(Into::into)
    }
}
