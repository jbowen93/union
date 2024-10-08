use std::marker::PhantomData;

use chain_utils::{ChainNotFoundError, GetChain};
use macros::apply;
use queue_msg::{
    data, defer_absolute, fetch, noop, now, queue_msg, seq, wait, HandleWait, Op, QueueError,
    QueueMessage,
};
use tracing::{debug, instrument};
use unionlabs::{
    ibc::core::client::height::{Height, IsHeight},
    ics24::ClientStatePath,
    traits::{ChainIdOf, ClientState, HeightOf},
    QueryHeight,
};

use crate::{
    any_enum, any_lc,
    data::{AnyData, Data, LatestHeight},
    fetch::{AnyFetch, Fetch, FetchState},
    id, identified, AnyLightClientIdentified, ChainExt, DoFetchState, RelayMessage,
};

#[apply(any_enum)]
#[any = AnyWait]
pub enum Wait<Hc: ChainExt, Tr: ChainExt> {
    Height(WaitForHeight<Hc, Tr>),
    HeightRelative(WaitForHeightRelative<Hc, Tr>),
    Timestamp(WaitForTimestamp<Hc, Tr>),
    TrustedHeight(WaitForTrustedHeight<Hc, Tr>),
}

impl HandleWait<RelayMessage> for AnyLightClientIdentified<AnyWait> {
    #[instrument(skip_all, fields(chain_id = %self.chain_id()))]
    async fn handle(
        self,
        store: &<RelayMessage as QueueMessage>::Store,
    ) -> Result<Op<RelayMessage>, QueueError> {
        let wait = self;

        any_lc! {
            |wait| {
                Ok(store
                    .with_chain(&wait.chain_id, move |c| async move { wait.t.handle(&c).await })
                    .map_err(|e: ChainNotFoundError<Hc>| QueueError::Fatal(Box::new(e)))?
                    .await
                    .map_err(|e| QueueError::Fatal(Box::new(e)))?
                )
            }
        }
    }
}

impl<Hc, Tr> Wait<Hc, Tr>
where
    AnyLightClientIdentified<AnyWait>: From<identified!(Wait<Hc, Tr>)>,
    AnyLightClientIdentified<AnyData>: From<identified!(Data<Hc, Tr>)>,
    AnyLightClientIdentified<AnyFetch>: From<identified!(Fetch<Tr, Hc>)>,
    Hc: ChainExt + DoFetchState<Hc, Tr>,
    Tr: ChainExt,
{
    pub async fn handle(self, c: &Hc) -> Result<Op<RelayMessage>, WaitError> {
        match self {
            Wait::Height(WaitForHeight { height, __marker }) => {
                let chain_height = c.query_latest_height().await.unwrap();
                if chain_height >= height {
                    Ok(noop())
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new method on chain
                        defer_absolute(now() + 1),
                        wait(id::<Hc, Tr, _>(
                            c.chain_id(),
                            WaitForHeight { height, __marker },
                        )),
                    ]))
                }
            }
            // REVIEW: Perhaps remove, unused
            Wait::HeightRelative(WaitForHeightRelative {
                height,
                __marker: _,
            }) => {
                let chain_height = c.query_latest_height().await.unwrap();

                Ok(wait(id::<Hc, Tr, _>(
                    c.chain_id(),
                    WaitForHeight {
                        height: Height {
                            revision_number: chain_height.revision_number(),
                            revision_height: chain_height.revision_height() + height,
                        }
                        .into(),
                        __marker: PhantomData,
                    },
                )))
            }
            Wait::Timestamp(WaitForTimestamp {
                timestamp,
                __marker,
            }) => {
                let chain_ts = c.query_latest_timestamp().await.unwrap();

                if chain_ts >= timestamp {
                    // TODO: Figure out a way to fetch a height at a specific timestamp
                    Ok(data(id(
                        c.chain_id(),
                        LatestHeight::<Hc, Tr> {
                            height: c.query_latest_height().await.unwrap(),
                            __marker: PhantomData,
                        },
                    )))
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + chain.block_time()`? Would require a new method on chain
                        defer_absolute(now() + 1),
                        wait(id::<Hc, Tr, _>(
                            c.chain_id(),
                            WaitForTimestamp {
                                timestamp,
                                __marker,
                            }
                            .into(),
                        )),
                    ]))
                }
            }
            Wait::TrustedHeight(WaitForTrustedHeight {
                client_id,
                counterparty_client_id,
                counterparty_chain_id,
                height,
            }) => {
                let trusted_client_state =
                    Hc::query_unfinalized_trusted_client_state(c, client_id.clone())
                        .await
                        .map_err(|err| {
                            WaitError::FetchUnfinalizedTrustedClientState(Box::new(err))
                        })?;

                if trusted_client_state.height().revision_height() >= height.revision_height() {
                    debug!(
                        "client height reached ({} >= {})",
                        trusted_client_state.height(),
                        height
                    );

                    // the height has been reached, fetch the counterparty client state on `Tr` at the trusted height
                    Ok(fetch(id(
                        counterparty_chain_id,
                        FetchState::<Tr, Hc> {
                            at: QueryHeight::Specific(trusted_client_state.height()),
                            path: ClientStatePath {
                                client_id: counterparty_client_id.clone(),
                            }
                            .into(),
                        },
                    )))
                } else {
                    Ok(seq([
                        // REVIEW: Defer until `now + counterparty_chain.block_time()`? Would require a new method on chain
                        defer_absolute(now() + 1),
                        wait(id(
                            c.chain_id(),
                            WaitForTrustedHeight {
                                client_id,
                                height,
                                counterparty_client_id,
                                counterparty_chain_id,
                            },
                        )),
                    ]))
                }
            }
        }
    }
}

/// See docs on `FetchError` for more information about the design of this type.
#[derive(macros::Debug, thiserror::Error)]
pub enum WaitError {
    #[error("error fetching unfinalized trusted client state")]
    FetchUnfinalizedTrustedClientState(
        #[source] Box<dyn std::error::Error + Send + Sync + 'static>,
    ),
}

#[queue_msg]
pub struct WaitForHeight<Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub height: HeightOf<Hc>,
}

#[queue_msg]
pub struct WaitForHeightRelative<#[cover] Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub height: u64,
}

#[queue_msg]
pub struct WaitForTimestamp<#[cover] Hc: ChainExt, #[cover] Tr: ChainExt> {
    pub timestamp: i64,
}

/// Wait for the client `.client_id` on `Hc` to trust a height >= `.height`, returning the counterparty's client state at that height when it's reached.
#[queue_msg]
pub struct WaitForTrustedHeight<Hc: ChainExt, Tr: ChainExt> {
    /// The id of the client on `Hc` who's [`ClientState::height()`] we're waiting to be >= `.height`.
    pub client_id: Hc::ClientId,
    /// The id of the counterparty client on `Tr`, who's state will be fetched at [`ClientState::height()`] when `.client_id` on `Hc` trusts a height >= `.height`.
    pub counterparty_client_id: Tr::ClientId,
    pub counterparty_chain_id: ChainIdOf<Tr>,
    pub height: Tr::Height,
}
