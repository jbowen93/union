use macros::model;

use crate::{
    errors::{required, InvalidLength, MissingField},
    hash::H256,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            account_proof::{AccountProof, TryFromAccountProofError},
            storage_proof::{StorageProof, TryFromStorageProofError},
        },
    },
};

#[model(proto(raw(protos::union::ibc::lightclients::scroll::v1::Header), into, from))]
pub struct Header {
    pub l1_height: Height,
    pub l1_account_proof: AccountProof,
    pub l2_state_root: H256,
    pub l2_state_proof: StorageProof,
    pub last_batch_index: u64,
    pub last_batch_index_proof: StorageProof,
    pub l2_ibc_account_proof: AccountProof,
}

impl From<Header> for protos::union::ibc::lightclients::scroll::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            l1_height: Some(value.l1_height.into()),
            l1_account_proof: Some(value.l1_account_proof.into()),
            l2_state_root: value.l2_state_root.into(),
            l2_state_proof: Some(value.l2_state_proof.into()),
            last_batch_index: value.last_batch_index,
            last_batch_index_proof: Some(value.last_batch_index_proof.into()),
            l2_ibc_account_proof: Some(value.l2_ibc_account_proof.into()),
        }
    }
}

#[derive(Debug)]
pub enum TryFromHeaderError {
    MissingField(MissingField),
    L1AccountProof(TryFromAccountProofError),
    L2StateRoot(InvalidLength),
    L2StateProof(TryFromStorageProofError),
    LastBatchIndexProof(TryFromStorageProofError),
    L2IbcAccountProof(TryFromAccountProofError),
}

impl TryFrom<protos::union::ibc::lightclients::scroll::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::scroll::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_height: required!(value.l1_height)?.into(),
            l1_account_proof: required!(value.l1_account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L1AccountProof)?,
            l2_state_root: value
                .l2_state_root
                .try_into()
                .map_err(TryFromHeaderError::L2StateRoot)?,
            l2_state_proof: required!(value.l2_state_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2StateProof)?,
            last_batch_index: value.last_batch_index,
            last_batch_index_proof: required!(value.last_batch_index_proof)?
                .try_into()
                .map_err(TryFromHeaderError::LastBatchIndexProof)?,
            l2_ibc_account_proof: required!(value.l2_ibc_account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2IbcAccountProof)?,
        })
    }
}
