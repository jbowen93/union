pub use crate::altair::minimal::SYNC_COMMITTEE_SIZE;
pub use crate::altair::minimal::{
    AggregateAndProof, Attestation, AttesterSlashing, ContributionAndProof, HistoricalBatch,
    IndexedAttestation, PendingAttestation, SignedAggregateAndProof, SignedContributionAndProof,
    SyncAggregate, SyncCommittee, SyncCommitteeContribution, SyncCommitteeMessage,
};
pub use crate::bellatrix::minimal::Transaction;
pub use crate::bellatrix::minimal::{
    BYTES_PER_LOGS_BLOOM, MAX_BYTES_PER_TRANSACTION, MAX_EXTRA_DATA_BYTES,
    MAX_TRANSACTIONS_PER_PAYLOAD, PROPORTIONAL_SLASHING_MULTIPLIER_BELLATRIX,
};
use crate::capella;
use crate::capella::presets::Preset;
pub use crate::phase0::minimal::{
    EPOCHS_PER_HISTORICAL_VECTOR, EPOCHS_PER_SLASHINGS_VECTOR, ETH1_DATA_VOTES_BOUND,
    HISTORICAL_ROOTS_LIMIT, MAX_ATTESTATIONS, MAX_ATTESTER_SLASHINGS, MAX_DEPOSITS,
    MAX_PROPOSER_SLASHINGS, MAX_VALIDATORS_PER_COMMITTEE, MAX_VOLUNTARY_EXITS,
    SLOTS_PER_HISTORICAL_ROOT, VALIDATOR_REGISTRY_LIMIT,
};

pub use capella::*;

pub const MAX_WITHDRAWALS_PER_PAYLOAD: usize = 4;
pub const MAX_BLS_TO_EXECUTION_CHANGES: usize = 16;
pub const MAX_VALIDATORS_PER_WITHDRAWALS_SWEEP: usize = 16;

pub const PRESET: Preset = Preset {
    max_bls_to_execution_changes: MAX_BLS_TO_EXECUTION_CHANGES,
    max_withdrawals_per_payload: MAX_WITHDRAWALS_PER_PAYLOAD,
    max_validators_per_withdrawals_sweep: MAX_VALIDATORS_PER_WITHDRAWALS_SWEEP,
};

pub type LightClientUpdate =
    capella::LightClientUpdate<SYNC_COMMITTEE_SIZE, BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>;

pub type LightClientHeader = capella::LightClientHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>;

pub type ExecutionPayload = capella::ExecutionPayload<
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_WITHDRAWALS_PER_PAYLOAD,
>;

pub type ExecutionPayloadHeader =
    capella::ExecutionPayloadHeader<BYTES_PER_LOGS_BLOOM, MAX_EXTRA_DATA_BYTES>;

pub type BlindedBeaconBlock = capella::BlindedBeaconBlock<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_WITHDRAWALS_PER_PAYLOAD,
    MAX_BLS_TO_EXECUTION_CHANGES,
>;

pub type BlindedBeaconBlockBody = capella::BlindedBeaconBlockBody<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_WITHDRAWALS_PER_PAYLOAD,
    MAX_BLS_TO_EXECUTION_CHANGES,
>;

pub type SignedBlindedBeaconBlock = capella::SignedBlindedBeaconBlock<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_WITHDRAWALS_PER_PAYLOAD,
    MAX_BLS_TO_EXECUTION_CHANGES,
>;

pub type BeaconState = capella::BeaconState<
    SLOTS_PER_HISTORICAL_ROOT,
    HISTORICAL_ROOTS_LIMIT,
    ETH1_DATA_VOTES_BOUND,
    VALIDATOR_REGISTRY_LIMIT,
    EPOCHS_PER_HISTORICAL_VECTOR,
    EPOCHS_PER_SLASHINGS_VECTOR,
    MAX_VALIDATORS_PER_COMMITTEE,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
>;

pub type BeaconBlockBody = capella::BeaconBlockBody<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_WITHDRAWALS_PER_PAYLOAD,
    MAX_BLS_TO_EXECUTION_CHANGES,
>;

pub type BeaconBlock = capella::BeaconBlock<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_WITHDRAWALS_PER_PAYLOAD,
    MAX_BLS_TO_EXECUTION_CHANGES,
>;

pub type SignedBeaconBlock = capella::SignedBeaconBlock<
    MAX_PROPOSER_SLASHINGS,
    MAX_VALIDATORS_PER_COMMITTEE,
    MAX_ATTESTER_SLASHINGS,
    MAX_ATTESTATIONS,
    MAX_DEPOSITS,
    MAX_VOLUNTARY_EXITS,
    SYNC_COMMITTEE_SIZE,
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_WITHDRAWALS_PER_PAYLOAD,
    MAX_BLS_TO_EXECUTION_CHANGES,
>;

pub type NoOpExecutionEngine = capella::NoOpExecutionEngine<
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_WITHDRAWALS_PER_PAYLOAD,
>;

pub type MockExecutionEngine<F> = capella::MockExecutionEngine<
    BYTES_PER_LOGS_BLOOM,
    MAX_EXTRA_DATA_BYTES,
    MAX_BYTES_PER_TRANSACTION,
    MAX_TRANSACTIONS_PER_PAYLOAD,
    MAX_WITHDRAWALS_PER_PAYLOAD,
    F,
>;
