use std::collections::VecDeque;

use frunk::{hlist_pat, HList};
use macros::apply;
use queue_msg::{
    aggregation::{do_aggregate, UseAggregate},
    fetch, queue_msg, HandleAggregate, Op, QueueError, QueueMessage,
};
use tracing::instrument;
use unionlabs::ibc::core::client::height::IsHeight;

use crate::{
    any_chain, any_enum,
    data::{AnyData, LatestHeight},
    fetch::{AnyFetch, Fetch, FetchBlockRange},
    id, AnyChainIdentified, BlockMessage, ChainExt, DoAggregate, Identified, IsAggregateData,
};

#[apply(any_enum)]
#[any = AnyAggregate]
#[specific = ChainSpecificAggregate]
pub enum Aggregate<C: ChainExt> {
    FetchBlockRange(AggregateFetchBlockRange<C>),
    #[serde(untagged)]
    ChainSpecific(ChainSpecificAggregate<C>),
}

impl HandleAggregate<BlockMessage> for AnyChainIdentified<AnyAggregate> {
    #[instrument(skip_all, fields(chain_id = %self.chain_id()))]
    fn handle(
        self,
        data: VecDeque<<BlockMessage as QueueMessage>::Data>,
    ) -> Result<Op<BlockMessage>, QueueError> {
        let aggregate = self;

        any_chain! {
            |aggregate| Ok(aggregate.handle(data))
        }
    }
}

impl<C: ChainExt> Identified<C, Aggregate<C>> {
    pub fn handle(self, data: VecDeque<AnyChainIdentified<AnyData>>) -> Op<BlockMessage>
    where
        Identified<C, C::Aggregate>: DoAggregate,

        Identified<C, LatestHeight<C>>: IsAggregateData,

        AnyChainIdentified<AnyFetch>: From<Identified<C, Fetch<C>>>,
    {
        let chain_id = self.chain_id;

        match self.t {
            Aggregate::ChainSpecific(ChainSpecificAggregate(aggregate)) => {
                <Identified<_, C::Aggregate> as DoAggregate>::do_aggregate(
                    id(chain_id, aggregate),
                    data,
                )
            }
            Aggregate::FetchBlockRange(aggregate) => do_aggregate(id(chain_id, aggregate), data),
        }
    }
}

#[queue_msg]
pub struct ChainSpecificAggregate<C: ChainExt>(pub C::Aggregate);

#[queue_msg]
pub struct AggregateFetchBlockRange<C: ChainExt> {
    pub from_height: C::Height,
}

impl<C: ChainExt> UseAggregate<BlockMessage> for Identified<C, AggregateFetchBlockRange<C>>
where
    Identified<C, LatestHeight<C>>: IsAggregateData,

    AnyChainIdentified<AnyFetch>: From<Identified<C, Fetch<C>>>,
{
    type AggregatedData = HList![Identified<C, LatestHeight<C>>];

    fn aggregate(
        Identified {
            chain_id: this_chain_id,
            t: AggregateFetchBlockRange { from_height },
        }: Self,
        hlist_pat![Identified {
            chain_id: latest_height_chain_id,
            t: LatestHeight(to_height),
        }]: Self::AggregatedData,
    ) -> Op<BlockMessage> {
        assert!(to_height.revision_height() > from_height.revision_number());
        assert_eq!(this_chain_id, latest_height_chain_id);

        fetch(Identified::<C, _>::new(
            this_chain_id,
            FetchBlockRange {
                from_height,
                to_height,
            },
        ))
    }
}
