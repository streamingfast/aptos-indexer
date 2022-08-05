#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TransactionStatistic {
    #[prost(uint64, tag="1")]
    pub count: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockMetadataWrapper {
    #[prost(message, optional, tag="1")]
    pub metadata: ::core::option::Option<BlockMetadata>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockMetadata {
    #[prost(string, tag="1")]
    pub id: ::prost::alloc::string::String,
    #[prost(uint64, tag="2")]
    pub round: u64,
    #[prost(message, optional, tag="3")]
    pub timestamp: ::core::option::Option<::prost_types::Timestamp>,
}
