use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, HexBinary, IbcEndpoint, StdError, Uint128, Uint256};
use ethabi::{ParamType, Token};

pub type GenericAck = Result<Binary, Binary>;

#[derive(thiserror::Error, Debug)]
pub enum EncodingError {
    #[error("ICS20 can handle a single coin only.")]
    Ics20OnlyOneCoin,
    #[error("Could not decode UCS01 packet: value: {data}, err: {err:?}", data = serde_utils::to_hex(.value))]
    InvalidUCS01PacketEncoding { value: Vec<u8>, err: ethabi::Error },
    #[error("Could not decode UCS01 ack, expected a boolean, got: {data}", data = serde_utils::to_hex(.got))]
    InvalidUCS01AckEncoding { got: Vec<u8> },
    #[error("Could not decode ICS20 packet: value: {data}, err: {err}", data = serde_utils::to_hex(.value))]
    InvalidICS20PacketEncoding { value: Vec<u8>, err: StdError },
    #[error("Could not decode ICS20 ack: value: {data}, err: {err}", data = serde_utils::to_hex(.value))]
    InvalidICS20AckEncoding { value: Vec<u8>, err: StdError },
    #[error("Invalid sender address: sender: {data}, err: {err}", data = serde_utils::to_hex(.value))]
    InvalidSender { value: String, err: StdError },
    #[error("Invalid receiver address: receiver: {data}, err: {err}", data = serde_utils::to_hex(.value))]
    InvalidReceiver { value: String, err: StdError },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransferPacketCommon<T> {
    pub sender: String,
    pub receiver: String,
    pub tokens: Vec<TransferToken>,
    pub extension: T,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TransferToken {
    pub denom: String,
    pub amount: Uint128,
}

impl From<Coin> for TransferToken {
    fn from(value: Coin) -> Self {
        Self {
            denom: value.denom,
            amount: value.amount,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ucs01TransferPacket {
    /// the sender address
    sender: HexBinary,
    /// the recipient address on the destination chain
    receiver: HexBinary,
    /// the transferred tokens
    tokens: Vec<TransferToken>,
    pub memo: String,
}

impl Ucs01TransferPacket {
    pub fn sender(&self) -> &HexBinary {
        &self.sender
    }

    pub fn receiver(&self) -> &HexBinary {
        &self.receiver
    }

    pub fn tokens(&self) -> &Vec<TransferToken> {
        &self.tokens
    }

    pub fn new(
        sender: HexBinary,
        receiver: HexBinary,
        tokens: Vec<TransferToken>,
        memo: String,
    ) -> Self {
        Self {
            sender,
            receiver,
            tokens,
            memo,
        }
    }
}

impl TryFrom<Ucs01TransferPacket> for Binary {
    type Error = EncodingError;

    fn try_from(value: Ucs01TransferPacket) -> Result<Binary, Self::Error> {
        Ok(ethabi::encode(&[
            Token::Bytes(value.sender.into()),
            Token::Bytes(value.receiver.into()),
            Token::Array(
                value
                    .tokens
                    .into_iter()
                    .map(|TransferToken { denom, amount }| {
                        Token::Tuple(vec![
                            Token::String(denom),
                            Token::Uint(Uint256::from(amount).to_be_bytes().into()),
                        ])
                    })
                    .collect(),
            ),
            Token::String(value.memo),
        ])
        .into())
    }
}

impl TryFrom<Binary> for Ucs01TransferPacket {
    type Error = EncodingError;

    fn try_from(value: Binary) -> Result<Self, Self::Error> {
        let encoded_packet = ethabi::decode(
            &[
                ParamType::Bytes,
                ParamType::Bytes,
                ParamType::Array(Box::new(ParamType::Tuple(vec![
                    ParamType::String,
                    ParamType::Uint(128),
                ]))),
                ParamType::String,
            ],
            &value,
        )
        .map_err(|err| EncodingError::InvalidUCS01PacketEncoding {
            value: value.to_vec(),
            err,
        })?;
        // NOTE: at this point, it is technically impossible to have any other branch than the one we
        // match unless there is a bug in the underlying `ethabi` crate
        match &encoded_packet[..] {
            [Token::Bytes(sender), Token::Bytes(receiver), Token::Array(tokens), Token::String(memo)] => {
                Ok(Ucs01TransferPacket {
                    sender: sender.clone().into(),
                    receiver: receiver.clone().into(),
                    tokens: tokens
                        .iter()
                        .map(|encoded_token| {
                            if let Token::Tuple(encoded_token_inner) = encoded_token {
                                match &encoded_token_inner[..] {
                                    [Token::String(denom), Token::Uint(amount)] => TransferToken {
                                        denom: denom.clone(),
                                        amount: Uint128::new(amount.as_u128()),
                                    },
                                    _ => unreachable!(),
                                }
                            } else {
                                unreachable!()
                            }
                        })
                        .collect(),
                    memo: memo.clone(),
                })
            }
            _ => unreachable!(),
        }
    }
}

// https://github.com/cosmos/ibc/tree/0cd8028ea593a240723d13bba17f3d61d62397ad/spec/app/ics-020-fungible-token-transfer#data-structures
// https://github.com/cosmos/ibc-go/blob/d02ab9db8fc80eb5e55041d3d6416370c33441f7/proto/ibc/applications/transfer/v2/packet.proto
#[cw_serde]
pub struct Ics20Packet {
    pub denom: String,
    pub amount: Uint128,
    pub sender: String,
    pub receiver: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub memo: String,
}

impl TryFrom<Ics20Packet> for Binary {
    type Error = EncodingError;
    fn try_from(value: Ics20Packet) -> Result<Binary, Self::Error> {
        Ok(cosmwasm_std::to_json_vec(&value)
            .expect("impossible")
            .into())
    }
}

impl TryFrom<Binary> for Ics20Packet {
    type Error = EncodingError;
    fn try_from(value: Binary) -> Result<Self, Self::Error> {
        cosmwasm_std::from_json(&value).map_err(|err| EncodingError::InvalidICS20PacketEncoding {
            value: value.to_vec(),
            err,
        })
    }
}

pub trait TransferPacket {
    type Extension: Into<String> + Clone;
    type Addr: ToString;

    // NOTE: can't ref here because cosmwasm_std::Coins don't impl iterator nor
    // exposes the underlying BTreeMap...
    fn tokens(&self) -> Vec<TransferToken>;

    fn sender(&self) -> &Self::Addr;

    fn receiver(&self) -> &Self::Addr;

    fn extension(&self) -> &Self::Extension;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct NoExtension;

impl From<NoExtension> for String {
    fn from(_: NoExtension) -> Self {
        String::new()
    }
}

impl TransferPacket for Ucs01TransferPacket {
    type Extension = String;
    type Addr = HexBinary;

    fn tokens(&self) -> Vec<TransferToken> {
        self.tokens().clone()
    }

    fn sender(&self) -> &HexBinary {
        &self.sender
    }

    fn receiver(&self) -> &HexBinary {
        &self.receiver
    }

    fn extension(&self) -> &Self::Extension {
        &self.memo
    }
}

impl TransferPacket for Ics20Packet {
    type Extension = String;
    type Addr = String;

    fn tokens(&self) -> Vec<TransferToken> {
        vec![TransferToken {
            denom: self.denom.clone(),
            amount: self.amount,
        }]
    }

    fn sender(&self) -> &String {
        &self.sender
    }

    fn receiver(&self) -> &String {
        &self.receiver
    }

    fn extension(&self) -> &Self::Extension {
        &self.memo
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Ucs01Ack {
    Failure,
    Success,
}

impl TryFrom<Ucs01Ack> for Binary {
    type Error = EncodingError;

    fn try_from(value: Ucs01Ack) -> Result<Self, Self::Error> {
        Ok(match value {
            Ucs01Ack::Failure => [0].into(),
            Ucs01Ack::Success => [1].into(),
        })
    }
}

impl TryFrom<Binary> for Ucs01Ack {
    type Error = EncodingError;

    fn try_from(value: Binary) -> Result<Self, Self::Error> {
        match value.as_slice() {
            [0] => Ok(Ucs01Ack::Failure),
            [1] => Ok(Ucs01Ack::Success),
            _ => Err(EncodingError::InvalidUCS01AckEncoding {
                got: value.to_vec(),
            }),
        }
    }
}

impl From<Ucs01Ack> for GenericAck {
    fn from(value: Ucs01Ack) -> Self {
        match value {
            Ucs01Ack::Failure => Err([0].into()),
            Ucs01Ack::Success => Ok([1].into()),
        }
    }
}

/// Standard ICS20 acknowledgement https://github.com/cosmos/cosmos-sdk/blob/v0.42.0/proto/ibc/core/channel/v1/channel.proto#L141-L147
#[cw_serde]
pub enum Ics20Ack {
    Result(Binary),
    Error(String),
}

impl TryFrom<Ics20Ack> for Binary {
    type Error = EncodingError;
    fn try_from(value: Ics20Ack) -> Result<Self, Self::Error> {
        Ok(cosmwasm_std::to_json_vec(&value)
            .expect("impossible")
            .into())
    }
}

impl TryFrom<Binary> for Ics20Ack {
    type Error = EncodingError;
    // Interesting, the Error variant of the enum clash with the AT in the return type, https://github.com/rust-lang/rust/issues/57644
    fn try_from(value: Binary) -> Result<Self, <Self as TryFrom<Binary>>::Error> {
        cosmwasm_std::from_json::<Ics20Ack>(&value).map_err(|err| {
            EncodingError::InvalidICS20AckEncoding {
                value: value.to_vec(),
                err,
            }
        })
    }
}

impl From<Ics20Ack> for GenericAck {
    fn from(value: Ics20Ack) -> Self {
        match value {
            Ics20Ack::Result(_) => Ok(value.try_into().unwrap()),
            Ics20Ack::Error(_) => Err(value.try_into().unwrap()),
        }
    }
}

impl TryFrom<TransferPacketCommon<String>> for Ics20Packet {
    type Error = EncodingError;

    fn try_from(
        TransferPacketCommon {
            sender,
            receiver,
            tokens,
            extension,
        }: TransferPacketCommon<String>,
    ) -> Result<Self, Self::Error> {
        let (denom, amount) = match &tokens[..] {
            [TransferToken { denom, amount }] => Ok((denom.clone(), *amount)),
            _ => Err(EncodingError::Ics20OnlyOneCoin),
        }?;
        Ok(Self {
            sender,
            receiver,
            denom,
            amount,
            memo: extension,
        })
    }
}

// https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md#data-structures
// SPEC: {ics20Port}/{ics20Channel}/{denom}
pub fn make_foreign_denom(endpoint: &IbcEndpoint, denom: &str) -> String {
    format!("{}/{}/{}", endpoint.port_id, endpoint.channel_id, denom)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenomOrigin<'a> {
    Local { denom: &'a str },
    Remote { denom: &'a str },
}

impl<'a> From<(&'a str, &IbcEndpoint)> for DenomOrigin<'a> {
    fn from((denom, remote_endpoint): (&'a str, &IbcEndpoint)) -> Self {
        // https://github.com/cosmos/ibc/blob/main/spec/app/ics-020-fungible-token-transfer/README.md#data-structures
        // SPEC: {ics20Port}/{ics20Channel}/{denom}
        // The denom is local IFF we can strip all prefixes
        match denom
            .strip_prefix(&remote_endpoint.port_id)
            .and_then(|denom| denom.strip_prefix('/'))
            .and_then(|denom| denom.strip_prefix(&remote_endpoint.channel_id))
            .and_then(|denom| denom.strip_prefix('/'))
        {
            Some(denom) => DenomOrigin::Local { denom },
            None => DenomOrigin::Remote { denom },
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{Binary, IbcEndpoint, Uint128};

    use super::{Ics20Packet, TransferToken, Ucs01Ack, Ucs01TransferPacket};
    use crate::types::{DenomOrigin, Ics20Ack};

    #[test]
    fn ucs01_packet_encode_decode_iso() {
        let packet = Ucs01TransferPacket {
            sender: b"a".into(),
            receiver: b"b".into(),
            tokens: vec![
                TransferToken {
                    denom: "denom-0".into(),
                    amount: Uint128::from(1u32),
                },
                TransferToken {
                    denom: "denom-1".into(),
                    amount: Uint128::MAX,
                },
                TransferToken {
                    denom: "denom-2".into(),
                    amount: Uint128::from(1337u32),
                },
            ],
            memo: String::new(),
        };
        assert_eq!(
            packet,
            Binary::try_from(packet.clone())
                .unwrap()
                .try_into()
                .unwrap()
        );
    }

    #[test]
    fn ucs01_ack_encode_decode_iso() {
        assert_eq!(
            Ucs01Ack::Success,
            Binary::try_from(Ucs01Ack::Success)
                .unwrap()
                .try_into()
                .unwrap()
        );
        assert_eq!(
            Ucs01Ack::Failure,
            Binary::try_from(Ucs01Ack::Failure)
                .unwrap()
                .try_into()
                .unwrap()
        );
    }

    #[test]
    fn ics20_packet_encode_decode_iso() {
        let packet = Ics20Packet {
            denom: "a".into(),
            amount: Uint128::from(1337u32),
            sender: "c".into(),
            receiver: "d".into(),
            memo: "bla".into(),
        };
        assert_eq!(
            packet,
            Binary::try_from(packet.clone())
                .unwrap()
                .try_into()
                .unwrap()
        );
    }

    #[test]
    fn ics20_ack_encode_decode_iso() {
        assert_eq!(
            Ics20Ack::Result(b"blabla".into()),
            Binary::try_from(Ics20Ack::Result(b"blabla".into()))
                .unwrap()
                .try_into()
                .unwrap()
        );
        assert_eq!(
            Ics20Ack::Error("ok".into()),
            Binary::try_from(Ics20Ack::Error("ok".into()))
                .unwrap()
                .try_into()
                .unwrap()
        );
    }

    #[test]
    fn denom_origin_parse_local() {
        assert_eq!(
            DenomOrigin::from((
                "port-1433/channel-44/token-k",
                &IbcEndpoint {
                    port_id: "port-1433".into(),
                    channel_id: "channel-44".into(),
                }
            )),
            DenomOrigin::Local { denom: "token-k" }
        );
    }

    #[test]
    fn denom_origin_parse_remote() {
        assert_eq!(
            DenomOrigin::from((
                "blabla/ok/-k",
                &IbcEndpoint {
                    port_id: "port-1433".into(),
                    channel_id: "channel-44".into(),
                }
            )),
            DenomOrigin::Remote {
                denom: "blabla/ok/-k"
            }
        );
    }
}
