// @generated
/// Module is the config object of the capability module.
#[cfg_attr(
    feature = "ethers",
    derive(::ethers::contract::EthAbiType, ::ethers::contract::EthAbiCodec)
)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Module {
    /// seal_keeper defines if keeper.Seal() will run on BeginBlock() to prevent further modules from creating a scoped
    /// keeper. For more details check x/capability/keeper.go.
    #[prost(bool, tag = "1")]
    pub seal_keeper: bool,
}
// @@protoc_insertion_point(module)
