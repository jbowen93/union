// @generated
/// PubKey is an bn254 public key for handling Tendermint keys in SDK.
/// It's needed for Any serialization and SDK compatibility.
/// It must not be used in a non Tendermint key context because it doesn't implement
/// ADR-28. Nevertheless, you will like to use bn254 in app user level
/// then you must create a new proto message and follow ADR-28 for Address construction.
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PubKey {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
}
/// Deprecated: PrivKey defines a bn254 private key.
/// NOTE: bn254 keys must not be used in SDK apps except in a tendermint validator context.
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PrivKey {
    #[prost(bytes = "vec", tag = "1")]
    pub key: ::prost::alloc::vec::Vec<u8>,
}
// @@protoc_insertion_point(module)
