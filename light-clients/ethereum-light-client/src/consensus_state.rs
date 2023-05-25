use crate::{errors::Error, eth_types::SyncCommittee};
use ethereum_verifier::{
    crypto::{eth_aggregate_public_keys, BlsPublicKey},
    primitives::Slot,
};
use ibc::core::ics23_commitment::commitment::CommitmentRoot;
use prost::Message;
use protos::{
    google::protobuf::Any as IBCAny,
    ibc::lightclients::tendermint::v1::ConsensusState as RawTmConsensusState,
    union::ibc::lightclients::{
        cometbls::v1::ConsensusState as RawCometConsensusState,
        ethereum::v1::ConsensusState as RawConsensusState,
    },
};

pub const ETHEREUM_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.lightclients.ethereum.v1.ConsensusState";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ConsensusState {
    /// finalized header's slot
    pub slot: Slot,
    /// the storage root of the IBC contract
    pub storage_root: CommitmentRoot,
    /// timestamp from execution payload
    pub timestamp: u64,
    /// aggregate public key of current sync committee
    pub current_sync_committee: BlsPublicKey,
    /// aggregate public key of next sync committee
    pub next_sync_committee: Option<BlsPublicKey>,
}

impl Default for ConsensusState {
    fn default() -> Self {
        Self {
            slot: Default::default(),
            storage_root: CommitmentRoot::from_bytes(Default::default()),
            timestamp: Default::default(),
            current_sync_committee: Default::default(),
            next_sync_committee: Default::default(),
        }
    }
}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = Error;

    fn try_from(value: RawConsensusState) -> Result<Self, Self::Error> {
        let next_sync_committee = if value.next_sync_committee.is_empty() {
            None
        } else {
            Some(
                BlsPublicKey::try_from(value.next_sync_committee.as_slice())
                    .map_err(|_| Error::InvalidPublicKey)?,
            )
        };
        Ok(Self {
            slot: value.slot,
            storage_root: value.storage_root.into(),
            timestamp: value.timestamp,
            current_sync_committee: BlsPublicKey::try_from(value.current_sync_committee.as_slice())
                .map_err(|_| Error::InvalidPublicKey)?,
            next_sync_committee,
        })
    }
}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        let next_sync_committee = match value.next_sync_committee {
            Some(next_sync_committee) => next_sync_committee.to_vec(),
            None => Vec::new(),
        };
        Self {
            slot: value.slot,
            storage_root: value.storage_root.into_vec(),
            timestamp: value.timestamp,
            current_sync_committee: value.current_sync_committee.to_vec(),
            next_sync_committee,
        }
    }
}

impl TryFrom<IBCAny> for ConsensusState {
    type Error = Error;

    fn try_from(raw: IBCAny) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            ETHEREUM_CONSENSUS_STATE_TYPE_URL => RawConsensusState::decode(raw.value.as_slice())
                .map_err(|_| Error::decode("when decoding proto consensus state (Any)"))?
                .try_into()
                .map_err(|_| Error::decode("when converting to consensus state (Any)")),
            _ => Err(Error::UnknownTypeUrl),
        }
    }
}

pub fn tendermint_to_cometbls_consensus_state(
    state: RawTmConsensusState,
) -> RawCometConsensusState {
    RawCometConsensusState {
        timestamp: state.timestamp,
        root: state.root,
        next_validators_hash: state.next_validators_hash,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrustedConsensusState {
    pub state: ConsensusState,
    pub current_sync_committee: Option<SyncCommittee>,
    pub next_sync_committee: Option<SyncCommittee>,
}

impl TrustedConsensusState {
    pub fn new(
        consensus_state: ConsensusState,
        sync_committee: SyncCommittee,
        is_next: bool,
    ) -> Result<Self, Error> {
        if eth_aggregate_public_keys(&sync_committee.public_keys)
            .map_err(|_| Error::InvalidSyncCommittee)?
            != sync_committee.aggregate_public_key
        {
            return Err(Error::InvalidSyncCommittee);
        }

        if !is_next {
            return if sync_committee.aggregate_public_key == consensus_state.current_sync_committee
            {
                Ok(Self {
                    state: consensus_state,
                    current_sync_committee: Some(sync_committee),
                    next_sync_committee: None,
                })
            } else {
                Err(Error::InvalidSyncCommittee)
            };
        }

        if let Some(next_sync_committee) = consensus_state.next_sync_committee.clone() {
            if sync_committee.aggregate_public_key == next_sync_committee {
                Ok(Self {
                    state: consensus_state,
                    current_sync_committee: None,
                    next_sync_committee: Some(sync_committee),
                })
            } else {
                Err(Error::InvalidSyncCommittee)
            }
        } else {
            Err(Error::NoNextSyncCommittee)
        }
    }

    pub fn current_sync_committee_aggregate_key(&self) -> BlsPublicKey {
        self.state.current_sync_committee.clone()
    }
}
