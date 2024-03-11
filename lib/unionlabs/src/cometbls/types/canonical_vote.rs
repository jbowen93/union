use macros::proto;
use serde::{Deserialize, Serialize};

use crate::{
    bounded::BoundedI64,
    tendermint::types::{canonical_block_id::CanonicalBlockId, signed_msg_type::SignedMsgType},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::tendermint::types::CanonicalVote, from)]
pub struct CanonicalVote {
    /// type alias for byte
    pub ty: SignedMsgType,
    /// canonicalization requires fixed size encoding here
    pub height: BoundedI64<0, { i64::MAX }>,
    /// canonicalization requires fixed size encoding here
    pub round: BoundedI64<0, { i64::MAX }>,
    pub block_id: CanonicalBlockId,
    pub chain_id: String,
}

impl From<CanonicalVote> for protos::tendermint::types::CanonicalVote {
    fn from(value: CanonicalVote) -> Self {
        Self {
            r#type: value.ty.into(),
            height: value.height.into(),
            round: value.round.into(),
            block_id: Some(value.block_id.into()),
            chain_id: value.chain_id,
        }
    }
}
