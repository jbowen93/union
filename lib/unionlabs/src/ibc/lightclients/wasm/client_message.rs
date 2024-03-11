use macros::proto;
use serde::{Deserialize, Serialize};

use crate::encoding::{Decode, DecodeErrorOf, Encode, Proto};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[proto(raw = protos::ibc::lightclients::wasm::v1::ClientMessage, into, from)]
pub struct ClientMessage<Data> {
    pub data: Data,
}

impl<Data: Encode<Proto>> From<ClientMessage<Data>>
    for protos::ibc::lightclients::wasm::v1::ClientMessage
{
    fn from(value: ClientMessage<Data>) -> Self {
        Self {
            data: value.data.encode(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromClientMessageError<Data: Decode<Proto>> {
    Data(DecodeErrorOf<Proto, Data>),
}

impl<Data: Decode<Proto>> TryFrom<protos::ibc::lightclients::wasm::v1::ClientMessage>
    for ClientMessage<Data>
{
    type Error = TryFromClientMessageError<Data>;

    fn try_from(
        value: protos::ibc::lightclients::wasm::v1::ClientMessage,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            data: Data::decode(&value.data).map_err(TryFromClientMessageError::Data)?,
        })
    }
}
