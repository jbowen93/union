use custom_debug_derive::Debug;
use macros::model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
// REVIEW: Are these fields fixed size?
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[model(proto(raw(protos::union::galois::api::v2::ZeroKnowledgeProof), into, from))]
pub struct ZeroKnowledgeProof {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(with = "::serde_utils::fmt::hex")]
    pub content: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(with = "::serde_utils::fmt::hex")]
    pub compressed_content: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(with = "::serde_utils::fmt::hex")]
    pub evm_proof: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(with = "::serde_utils::fmt::hex")]
    pub public_inputs: Vec<u8>,
}

impl From<ZeroKnowledgeProof> for protos::union::galois::api::v2::ZeroKnowledgeProof {
    fn from(value: ZeroKnowledgeProof) -> Self {
        Self {
            content: value.content,
            compressed_content: value.compressed_content,
            evm_proof: value.evm_proof,
            public_inputs: value.public_inputs,
        }
    }
}

impl From<protos::union::galois::api::v2::ZeroKnowledgeProof> for ZeroKnowledgeProof {
    fn from(value: protos::union::galois::api::v2::ZeroKnowledgeProof) -> Self {
        Self {
            content: value.content,
            compressed_content: value.compressed_content,
            evm_proof: value.evm_proof,
            public_inputs: value.public_inputs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde() {
        let json = serde_json::to_string_pretty(&ZeroKnowledgeProof {
            content: [].into(),
            compressed_content: [].into(),
            evm_proof: [].into(),
            public_inputs: [].into(),
        })
        .unwrap();

        println!("{json}");
    }
}
