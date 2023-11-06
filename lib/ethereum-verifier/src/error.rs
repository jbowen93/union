use milagro_bls::AmclError;
use trie_db::TrieError;
use unionlabs::ethereum::H256;

#[derive(Debug, PartialEq)]
pub struct InvalidMerkleBranch {
    pub leaf: H256,
    pub branch: Vec<H256>,
    pub depth: usize,
    pub index: u64,
    pub root: H256,
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("invalid merkle branch ({0:?})")]
    InvalidMerkleBranch(InvalidMerkleBranch),
    #[error("invalid chain conversion")]
    InvalidChainVersion,
    #[error("crypto error")]
    Crypto,
    #[error(
        "expected current sync committee to be provided since `update_period == current_period`"
    )]
    ExpectedCurrentSyncCommittee,
    #[error("expected next sync committee to be provided since `update_period > current_period`")]
    ExpectedNextSyncCommittee,
    #[error(
        "irrelevant update since the order of the slots in the update data, and stored data is not correct"
    )]
    IrrelevantUpdate,
    #[error("the order of the slots in the update data, and stored data is not correct")]
    InvalidSlots,
    #[error(
        "signature period must be equal to `store_period` or `store_period + 1` \
                when the next sync committee is stored. Otherwise, it must be equal to `store_period`"
    )]
    InvalidSignaturePeriod,
    #[error("signature is not valid")]
    InvalidSignature,
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("next sync committee does not match with the one in the current state")]
    NextSyncCommitteeMismatch,
    #[error(
        "insufficient number of sync committee participants, expected it to be at least ({0}) but got ({1})",
    )]
    InsufficientSyncCommitteeParticipants(usize, usize),
    #[error("bls error ({0:?})")]
    Bls(AmclError),
    #[error("proof is invalid due to value mismatch")]
    ValueMismatch,
    #[error("trie error ({0:?})")]
    Trie(Box<TrieError<primitive_types::H256, rlp::DecoderError>>),
    #[error("rlp decoding failed ({0})")]
    RlpDecode(String),
    #[error("custom query error: ({0})")]
    CustomError(String),
}

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("verify storage absence error: {0}")]
pub struct VerifyStorageAbsenceError(#[from] Error);

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("validate light client error: {0}")]
pub struct ValidateLightClientError(#[from] Error);

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("verify account storage root error: {0}")]
pub struct VerifyAccountStorageRootError(#[from] Error);

#[derive(Debug, thiserror::Error, PartialEq)]
#[error("verify storage proof error: {0}")]
pub struct VerifyStorageProofError(#[from] Error);

impl From<AmclError> for Error {
    fn from(e: AmclError) -> Self {
        Error::Bls(e)
    }
}

impl From<Box<TrieError<primitive_types::H256, rlp::DecoderError>>> for Error {
    fn from(e: Box<TrieError<primitive_types::H256, rlp::DecoderError>>) -> Self {
        Error::Trie(e)
    }
}
