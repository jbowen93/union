use serde::{Deserialize, Serialize};

use crate::{errors::UnknownEnumVariant, ibc::core::channel::order::Order};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct Version {
    // TODO(benluelo): "The identifier field specifies a unique version identifier. A value of "1" specifies IBC 1.0.0."
    pub identifier: String,
    pub features: Vec<Order>,
}

impl TryFrom<protos::ibc::core::connection::v1::Version> for Version {
    type Error = UnknownEnumVariant<String>;

    fn try_from(proto: protos::ibc::core::connection::v1::Version) -> Result<Self, Self::Error> {
        Ok(Self {
            identifier: proto.identifier,
            features: proto
                .features
                .into_iter()
                .map(|order| order.parse())
                .collect::<Result<_, _>>()?,
        })
    }
}

impl From<Version> for protos::ibc::core::connection::v1::Version {
    fn from(value: Version) -> Self {
        Self {
            identifier: value.identifier,
            features: value
                .features
                .into_iter()
                .map(|feature| <&'static str>::from(feature).to_string())
                .collect(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl From<Version> for contracts::ibc_handler::IbcCoreConnectionV1VersionData {
    fn from(value: Version) -> Self {
        Self {
            identifier: value.identifier,
            features: value
                .features
                .into_iter()
                .map(|order| <&'static str>::from(order).to_string())
                .collect(),
        }
    }
}

#[cfg(feature = "ethabi")]
impl TryFrom<contracts::ibc_handler::IbcCoreConnectionV1VersionData> for Version {
    type Error = UnknownEnumVariant<String>;

    fn try_from(
        value: contracts::ibc_handler::IbcCoreConnectionV1VersionData,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            identifier: value.identifier,
            features: value
                .features
                .into_iter()
                .map(|order| order.parse())
                .collect::<Result<_, _>>()?,
        })
    }
}
