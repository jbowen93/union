use unionlabs::cosmos::ics23::{
    hash_op::HashOp, leaf_op::LeafOp, length_op::LengthOp, proof_spec::ProofSpec,
};

use super::{hash_op, length_op, validate_iavl_ops};
use crate::{
    proof_specs::{self, IAVL_PROOF_SPEC},
    ValidateIavlOpsError,
};

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum SpecMismatchError {
    #[error("unexpected hash op ({0:?})")]
    UnexpectedHashOp(HashOp),
    #[error("unexpected prehash key ({0:?})")]
    UnexpectedPrehashKey(HashOp),
    #[error("unexpected prehash value ({0:?})")]
    UnexpectedPrehashValue(HashOp),
    #[error("unexpected length op ({0:?})")]
    UnexpectedLengthOp(LengthOp),
    #[error("bad prefix remaining {0} bytes after reading")]
    BadPrefix(usize),
    #[error("prefix ({prefix:?}) is not the prefix of ({full:?})")]
    PrefixMismatch { full: String, prefix: String },
    #[error("validate iavl ops ({0})")]
    ValidateIavlOps(ValidateIavlOpsError),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum ApplyError {
    #[error("key needed")]
    KeyNeeded,
    #[error("value needed")]
    ValueNeeded,
    #[error("apply leaf ({0:?})")]
    LeafData(super::length_op::ApplyError),
}

pub fn check_against_spec(leaf_op: &LeafOp, spec: &ProofSpec) -> Result<(), SpecMismatchError> {
    let lspec = &spec.leaf_spec;

    if proof_specs::compatible(spec, &IAVL_PROOF_SPEC) {
        match validate_iavl_ops(&leaf_op.prefix, 0) {
            Ok(remaining) => {
                if remaining > 0 {
                    return Err(SpecMismatchError::BadPrefix(remaining));
                }
            }
            Err(e) => return Err(SpecMismatchError::ValidateIavlOps(e)),
        }
    }

    if leaf_op.hash != lspec.hash {
        return Err(SpecMismatchError::UnexpectedHashOp(lspec.hash));
    }

    if leaf_op.prehash_key != lspec.prehash_key {
        return Err(SpecMismatchError::UnexpectedPrehashKey(lspec.prehash_key));
    }

    if leaf_op.prehash_value != lspec.prehash_value {
        return Err(SpecMismatchError::UnexpectedPrehashValue(
            lspec.prehash_value,
        ));
    }

    if leaf_op.length != lspec.length {
        return Err(SpecMismatchError::UnexpectedLengthOp(lspec.length));
    }

    if !leaf_op.prefix.starts_with(&lspec.prefix) {
        return Err(SpecMismatchError::PrefixMismatch {
            full: hex::encode(&leaf_op.prefix),
            prefix: hex::encode(&lspec.prefix),
        });
    }

    Ok(())
}

/// Calculate the leaf hash given the key and value being proven
pub fn apply(leaf_op: &LeafOp, key: &[u8], value: &[u8]) -> Result<Vec<u8>, ApplyError> {
    if key.is_empty() {
        return Err(ApplyError::KeyNeeded);
    }

    if value.is_empty() {
        return Err(ApplyError::ValueNeeded);
    }

    let pkey = prepare_data(leaf_op, leaf_op.prehash_key, key)?;

    let pvalue = prepare_data(leaf_op, leaf_op.prehash_value, value)?;

    let mut data = leaf_op.prefix.clone().into_owned();
    data.extend_from_slice(&pkey);
    data.extend_from_slice(&pvalue);

    Ok(hash_op::do_hash(leaf_op.hash, &data))
}

fn prepare_data(leaf_op: &LeafOp, hash_op: HashOp, data: &[u8]) -> Result<Vec<u8>, ApplyError> {
    let hdata = hash_op::do_hash_or_noop(hash_op, data);
    length_op::apply(&leaf_op.length, &hdata).map_err(ApplyError::LeafData)
}
