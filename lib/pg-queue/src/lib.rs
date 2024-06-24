use std::{
    borrow::Borrow,
    cmp::Eq,
    collections::HashMap,
    future::Future,
    hash::Hash,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use frame_support_procedural::{CloneNoBound, DebugNoBound};
use queue_msg::{
    optimize::{OptimizationResult, Pass, PurePass},
    Op, QueueMessage,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, prelude::FromRow, types::Json, Either, PgPool};
// use tokio_postgres::{types::Json, NoTls};
use tracing::{debug, info_span, trace, Instrument};

// pub static MIGRATOR: Migrator = sqlx::migrate!(); // defaults to "./migrations"

/// A fifo queue backed by a postgres table. Not suitable for high-throughput, but enough for ~1k items/sec.
///
/// The queue assumes the following database schema:
///
/// ```ignore
/// id SERIAL AUTO INCREMENT
/// status 0..2
/// item JSONB
/// error TEXT
/// ```
#[derive(DebugNoBound, CloneNoBound)]
pub struct PgQueue<T> {
    // lock: Arc<AtomicBool>,
    // client: Arc<tokio_postgres::Client>,
    client: PgPool,
    __marker: PhantomData<fn() -> T>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PgQueueConfig {
    pub database_url: String,
    pub max_connections: Option<u32>,
    pub min_connections: Option<u32>,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
}

impl PgQueueConfig {
    pub async fn into_pg_pool(self) -> sqlx::Result<PgPool> {
        PgPoolOptions::new()
            .max_connections(self.max_connections.unwrap_or(10))
            .min_connections(self.min_connections.unwrap_or(0))
            .idle_timeout(self.idle_timeout)
            .max_lifetime(self.max_lifetime)
            .connect(&self.database_url)
            .await
    }
}

#[derive(FromRow)]
struct Id {
    id: i64,
}

#[derive(Debug, FromRow)]
struct Record {
    id: i64,
    parents: Vec<i64>,
    item: String,
    created_at: sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>,
}

impl<T: QueueMessage> queue_msg::Queue<T> for PgQueue<T> {
    type Config = PgQueueConfig;
    // type Error = tokio_postgres::Error;
    type Error = sqlx::Error;

    async fn new(config: Self::Config) -> Result<Self, Self::Error> {
        // // Connect to the database.
        // let (client, connection) = tokio_postgres::connect(&config.database_url, NoTls).await?;

        // // The connection object performs the actual communication with the database,
        // // so spawn it off to run on its own.
        // tokio::spawn(async move {
        //     if let Err(e) = connection.await {
        //         eprintln!("connection error: {}", e);
        //     }
        // });

        Ok(Self {
            // lock: Arc::new(AtomicBool::new(false)),
            client: config.into_pg_pool().await?,
            __marker: PhantomData,
        })
    }

    async fn enqueue<'a, O: PurePass<T>>(
        &'a self,
        item: Op<T>,
        pre_enqueue_passes: &'a O,
    ) -> Result<(), Self::Error> {
        trace!("enqueue");

        let OptimizationResult {
            optimize_further,
            ready,
        } = pre_enqueue_passes.run_pass_pure(vec![item]);

        let mut tx = self.client.begin().await?;

        let ready_ids = sqlx::query(
            "
            INSERT INTO queue (item)
            SELECT * FROM UNNEST($1::JSONB[])
            RETURNING id
            ",
        )
        .bind(ready.into_iter().map(|x| Json(x.1)).collect::<Vec<_>>())
        .try_map(|x| Id::from_row(&x))
        .fetch_all(tx.as_mut())
        .await?;

        for ready in ready_ids {
            debug!(id = ready.id, "enqueued item");
        }

        let optimize_further_ids = sqlx::query(
            "
            INSERT INTO optimize (item)
            SELECT * FROM UNNEST($1::JSONB[])
            RETURNING id
            ",
        )
        .bind(
            optimize_further
                .into_iter()
                .map(|x| Json(x.1))
                .collect::<Vec<_>>(),
        )
        .try_map(|x| Id::from_row(&x))
        .fetch_all(tx.as_mut())
        .await?;

        for ready in optimize_further_ids {
            debug!(id = ready.id, "enqueued item");
        }

        tx.commit().await?;

        // self.lock.store(false, Ordering::SeqCst);

        Ok(())
    }

    async fn process<'a, F, Fut, R, O>(
        &'a self,
        pre_reenqueue_passes: &'a O,
        f: F,
    ) -> Result<Option<R>, Self::Error>
    where
        F: (FnOnce(Op<T>) -> Fut) + Send + 'static,
        Fut: Future<Output = (R, Result<Vec<Op<T>>, String>)> + Send + 'static,
        R: Send + Sync + 'static,
        O: PurePass<T>,
    {
        trace!("process");

        // if self.lock.swap(false, Ordering::SeqCst) {
        //     trace!("queue is locked");
        //     tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        // }

        let mut tx = self.client.begin().await?;

        let row = sqlx::query(
            r#"
            DELETE FROM
              queue
            WHERE
              id = (
                SELECT
                  id
                FROM
                  queue
                ORDER BY
                  id ASC
                FOR UPDATE
                  SKIP LOCKED
                LIMIT 1)
            RETURNING
              id,
              parents,
              item::text,
              created_at
            "#,
        )
        .try_map(|x| Record::from_row(&x))
        .fetch_optional(tx.as_mut())
        .await?;

        match row {
            Some(row) => {
                let span = info_span!("processing item", id = row.id);

                trace!(%row.item);

                // really don't feel like defining a new error type right now
                let json = de(&row.item).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

                let (r, res) = f(json).instrument(span).await;

                match res {
                    Err(error) => {
                        // Insert error message in the queue
                        sqlx::query(
                            r#"
                            INSERT INTO
                                failed (id, parents, item,      created_at, message)
                                VALUES ($1, $2,      $3::JSONB, $4,         $5     )
                            "#,
                        )
                        .bind(row.id)
                        .bind(row.parents)
                        .bind(row.item)
                        .bind(row.created_at)
                        .bind(error)
                        .execute(tx.as_mut())
                        .await?;
                        tx.commit().await?;
                    }
                    Ok(new_msgs) => {
                        sqlx::query(
                            "
                            INSERT INTO
                                done   (id, parents, item,      created_at)
                                VALUES ($1, $2,      $3::JSONB, $4        )
                            ",
                        )
                        .bind(row.id)
                        .bind(row.parents)
                        .bind(row.item)
                        .bind(row.created_at)
                        .execute(tx.as_mut())
                        .await?;

                        let OptimizationResult {
                            optimize_further,
                            ready,
                        } = pre_reenqueue_passes.run_pass_pure(new_msgs);

                        let ready_ids = sqlx::query(
                            "
                            INSERT INTO queue (item)
                            SELECT * FROM UNNEST($1::JSONB[])
                            RETURNING id
                            ",
                        )
                        .bind(ready.into_iter().map(|x| Json(x.1)).collect::<Vec<_>>())
                        .try_map(|x| Id::from_row(&x))
                        .fetch_all(tx.as_mut())
                        .await?;

                        for ready in ready_ids {
                            debug!(id = ready.id, "enqueued item");
                        }

                        let optimize_further_ids = sqlx::query(
                            "
                            INSERT INTO optimize (item)
                            SELECT * FROM UNNEST($1::JSONB[])
                            RETURNING id
                            ",
                        )
                        .bind(
                            optimize_further
                                .into_iter()
                                .map(|x| Json(x.1))
                                .collect::<Vec<_>>(),
                        )
                        .try_map(|x| Id::from_row(&x))
                        .fetch_all(tx.as_mut())
                        .await?;

                        for ready in optimize_further_ids {
                            debug!(id = ready.id, "enqueued item");
                        }

                        tx.commit().await?;
                    }
                }

                Ok(Some(r))
            }
            None => {
                // trace!("queue is empty");

                // self.lock.store(true, Ordering::SeqCst);
                // tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

                Ok(None)
            }
        }
    }

    async fn optimize<'a, O: Pass<T>>(
        &'a self,
        optimizer: &'a O,
    ) -> Result<(), Either<Self::Error, O::Error>> {
        trace!("optimize");

        // if self.lock.swap(false, Ordering::SeqCst) {
        //     debug!("queue is locked");
        //     tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        // }

        let mut tx = self.client.begin().await.map_err(Either::Left)?;

        let msgs = sqlx::query(
            r#"
            DELETE FROM
              optimize
            WHERE
              id = (
                SELECT
                  id
                FROM
                  optimize
                ORDER BY
                  id ASC
                FOR UPDATE
                  SKIP LOCKED
                LIMIT 1)
            RETURNING
              id,
              parents,
              item::text,
              created_at
            "#,
        )
        .try_map(|x| Record::from_row(&x))
        .fetch_all(tx.as_mut())
        .await
        .map_err(Either::Left)?;

        let (ids, msgs) = msgs
            .into_iter()
            .map(|r| {
                Ok((
                    r.id,
                    de(&r.item).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
                ))
            })
            .collect::<Result<(Vec<_>, Vec<_>), sqlx::Error>>()
            .map_err(Either::Left)?;

        let OptimizationResult {
            optimize_further,
            ready,
        } = optimizer
            .run_pass(msgs.clone())
            .instrument(info_span!(
                "optimizing items",
                ids = ids
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            ))
            .await
            .map_err(Either::Right)?;

        trace!(
            ready = ready.len(),
            optimize_further = optimize_further.len(),
            "optimized items"
        );

        let get_parent_ids = |parent_idxs: &[usize]| {
            ids.iter()
                .enumerate()
                .filter_map(|(idx, id)| parent_idxs.contains(&idx).then_some(*id))
                .collect::<Vec<_>>()
        };

        for (parent_idxs, new_msg) in optimize_further {
            let parents = get_parent_ids(&parent_idxs);
            debug!(parent_idxs = ?&parent_idxs, parents = ?&parents);

            let new_row = sqlx::query(
                "
                INSERT INTO queue (item, parents)
                VALUES
                    ($1::JSONB, $2)
                RETURNING id
                ",
            )
            .bind(Json(new_msg))
            .bind(&parents)
            .try_map(|x| Id::from_row(&x))
            .fetch_one(tx.as_mut())
            .await
            .map_err(Either::Left)?;

            debug!(id = new_row.id, "inserted new optimizer message");
        }

        for (parent_idxs, new_msg) in ready {
            let parents = get_parent_ids(&parent_idxs);
            debug!(parent_idxs = ?&parent_idxs, parents = ?&parents);

            let new_row = sqlx::query(
                "
                INSERT INTO queue (item, parents)
                VALUES
                    ($1::JSONB, $2)
                RETURNING id
                ",
            )
            .bind(Json(new_msg))
            .bind(&parents)
            .try_map(|x| Id::from_row(&x))
            .fetch_one(tx.as_mut())
            .await
            .map_err(Either::Left)?;

            debug!(id = new_row.id, "inserted new message");
        }

        tx.commit().await.map_err(Either::Left)?;

        Ok(())
    }

    // /// Enqueues a new item for processing.
    // async fn enqueue<'a, A>(
    //     &self,
    //     conn: A,
    //     item: T,
    //     parents: Vec<i64>,
    //     status: EnqueueStatus,
    // ) -> Result<(), sqlx::Error>
    // where
    //     A: Acquire<'a, Database = Postgres>,
    // {
    // }

    // /// Processes the next value from the queue, calling `f` on the value. Dequeueing has the following properties:
    // /// - if `f` returns an error, the item is requeued.
    // /// - if `f` returns Err(why), the item is permanently marked as failed with `why`.
    // /// - if `f` returns Ok(Some(msg)), the item is marked as processed and `msg` is queued.
    // /// - if `f` returns Ok(None), the item is marked as processed.
    // ///
    // /// Database atomicity is used to ensure that the queue is always in a consistent state, meaning that an item
    // /// process will always be retried until it reaches ProcessFlow::Fail or ProcessFlow::Success. `f` is responsible for
    // /// storing metadata in the job to determine if retrying should fail permanently.
    // ///
    // /// If the queue is not empty, then Some(R) will be returned, otherwise None.
    // pub async fn process<'a, 'b, F, Fut, R, A, P>(
    //     &self,
    //     conn: A,
    //     f: F,
    //     post_process: P,
    // ) -> Result<Option<R>, sqlx::Error>
    // where
    //     F: (FnOnce(T) -> Fut) + 'b,
    //     Fut: Future<Output = (R, Result<Vec<T>, String>)> + 'static,
    //     A: Acquire<'a, Database = Postgres>,
    //     // (optimize, ready)
    //     P: FnOnce(Vec<T>) -> (Vec<(Vec<usize>, T)>, Vec<(Vec<usize>, T)>),
    // {
    //     if self.lock.swap(false, Ordering::SeqCst) {
    //         trace!("queue is locked");
    //         tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    //     }

    //     let mut tx = conn.begin().await?;

    //     // #[derive(Debug)]
    //     // struct Record<T> {
    //     //     id: i64,
    //     //     item: String,
    //     // }

    //     let row = query!(
    //         r#"
    //         DELETE FROM
    //           queue
    //         WHERE
    //           id = (
    //             SELECT
    //               id
    //             FROM
    //               queue
    //             WHERE
    //               status = 'ready'::status
    //             ORDER BY
    //               id ASC
    //             FOR UPDATE
    //               SKIP LOCKED
    //             LIMIT 1)
    //         RETURNING
    //           id,
    //           parents,
    //           item::text AS "item!: String",
    //           created_at
    //         "#,
    //     )
    //     .fetch_optional(tx.as_mut())
    //     .await?;

    //     match row {
    //         Some(row) => {
    //             let span = info_span!("processing item", id = row.id);

    //             trace!(%row.item);

    //             // really don't feel like defining a new error type right now
    //             let json = de(&row.item).map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

    //             let (r, res) = f(json).instrument(span).await;

    //             match res {
    //                 Err(error) => {
    //                     // Insert error message in the queue
    //                     query!(
    //                         r#"
    //                         INSERT INTO
    //                           failed (id, parents, item, created_at, message)
    //                           VALUES ($1, $2,      $3,   $4,         $5     )
    //                         "#,
    //                         row.id,
    //                         row.parents,
    //                         row.item,
    //                         row.created_at,
    //                         error,
    //                     )
    //                     .execute(tx.as_mut())
    //                     .await?;
    //                     tx.commit().await?;
    //                 }
    //                 Ok(new_msgs) => {
    //                     let (optimize_further, ready) = post_process(new_msgs);

    //                     for (_parents, new_msg) in optimize_further {
    //                         self.enqueue(&mut tx, new_msg, vec![row.id], EnqueueStatus::Optimize)
    //                             .await?;
    //                     }

    //                     for (_parents, new_msg) in ready {
    //                         self.enqueue(&mut tx, new_msg, vec![row.id], EnqueueStatus::Ready)
    //                             .await?;
    //                     }

    //                     tx.commit().await?;
    //                 }
    //             }

    //             Ok(Some(r))
    //         }
    //         None => {
    //             trace!("queue is empty");

    //             self.lock.store(true, Ordering::SeqCst);
    //             tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

    //             Ok(None)
    //         }
    //     }
    // }

    // pub async fn optimize<'a, 'b, F, Fut, A, E>(
    //     &self,
    //     conn: A,
    //     f: F,
    // ) -> Result<(), Either<sqlx::Error, E>>
    // where
    //     F: (FnOnce(Vec<T>) -> Fut) + 'b,
    //     // (optimize, ready)
    //     Fut: Future<Output = Result<(Vec<(Vec<usize>, T)>, Vec<(Vec<usize>, T)>), E>> + 'b,
    //     A: Acquire<'a, Database = Postgres>,
    // {
    //     // if self.lock.swap(false, Ordering::SeqCst) {
    //     //     debug!("queue is locked");
    //     //     tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
    //     // }

    //     let mut tx = conn.begin().await.map_err(Either::Left)?;

    //     let msgs = sqlx::query!(
    //         r#"
    //         WITH _locked AS (
    //             SELECT
    //                 id
    //             FROM
    //                 queue
    //             WHERE
    //                 status = 'optimize'::status
    //             ORDER BY
    //                 id ASC
    //             FOR UPDATE
    //                 SKIP LOCKED)
    //         UPDATE
    //             queue
    //         SET
    //             status = 'done'::status
    //         WHERE
    //             id = ANY (
    //                 SELECT
    //                     id
    //                 FROM
    //                     _locked)
    //             RETURNING
    //                 id,
    //                 item::text AS "item!: String"
    //         "#,
    //     )
    //     .fetch_all(tx.as_mut())
    //     .await
    //     .map_err(Either::Left)?;

    //     let (ids, msgs) = msgs
    //         .into_iter()
    //         .map(|r| {
    //             Ok((
    //                 r.id,
    //                 de(&r.item).map_err(|e| sqlx::Error::Decode(Box::new(e)))?,
    //             ))
    //         })
    //         .collect::<Result<(Vec<_>, Vec<_>), sqlx::Error>>()
    //         .map_err(Either::Left)?;

    //     let span = info_span!(
    //         "optimizing items",
    //         ids = ids
    //             .iter()
    //             .map(|id| id.to_string())
    //             .collect::<Vec<_>>()
    //             .join(",")
    //     );
    //     let (optimize_further, ready) = f(msgs.clone())
    //         .instrument(span)
    //         .await
    //         .map_err(Either::Right)?;

    //     trace!(
    //         ready = ready.len(),
    //         optimize_further = optimize_further.len(),
    //         "optimized items"
    //     );

    //     let get_parent_ids = |parent_idxs: &[usize]| {
    //         ids.iter()
    //             .enumerate()
    //             .filter_map(|(idx, id)| parent_idxs.contains(&idx).then_some(*id))
    //             .collect::<Vec<_>>()
    //     };

    //     for (parent_idxs, new_msg) in optimize_further {
    //         let parents = get_parent_ids(&parent_idxs);
    //         debug!(parent_idxs = ?&parent_idxs, parents = ?&parents);

    //         let new_row = query!(
    //             "
    //             INSERT INTO queue (item, parents, status)
    //             VALUES
    //                 ($1::JSONB, $2, 'optimize') RETURNING id
    //             ",
    //             Json(new_msg) as _,
    //             &parents
    //         )
    //         .fetch_one(tx.as_mut())
    //         .await
    //         .map_err(Either::Left)?;

    //         debug!(id = new_row.id, "inserted new optimizer message");
    //     }

    //     for (parent_idxs, new_msg) in ready {
    //         let parents = get_parent_ids(&parent_idxs);
    //         debug!(parent_idxs = ?&parent_idxs, parents = ?&parents);

    //         let new_row = query!(
    //             "
    //             INSERT INTO queue (item, parents)
    //             VALUES
    //                 ($1::JSONB, $2) RETURNING id
    //             ",
    //             Json(new_msg) as _,
    //             &parents
    //         )
    //         .fetch_one(tx.as_mut())
    //         .await
    //         .map_err(Either::Left)?;

    //         debug!(id = new_row.id, "inserted new message");
    //     }

    //     tx.commit().await.map_err(Either::Left)?;

    //     Ok(())
    // }
}

#[derive(sqlx::Type)]
#[sqlx(type_name = "status", rename_all = "lowercase")]
pub enum EnqueueStatus {
    Ready,
    Optimize,
}

fn de<T: DeserializeOwned>(s: &str) -> Result<T, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(s);
    deserializer.disable_recursion_limit();
    // let deserializer = serde_stacker::Deserializer::new(&mut deserializer);
    let json = T::deserialize(&mut deserializer)?;
    Ok(json)
}

pub trait MapExt<K, V> {
    fn get_many<'a, Q>(&'a self, ks: impl IntoIterator<Item = &'a Q>) -> Vec<&'a V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + 'a;
}

impl<K: Hash + Eq, V> MapExt<K, V> for HashMap<K, V> {
    fn get_many<'a, Q>(&'a self, ks: impl IntoIterator<Item = &'a Q>) -> Vec<&'a V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq + 'a,
    {
        let mut out = vec![];

        for k in ks {
            out.extend(self.get(k));
        }

        out
    }
}
