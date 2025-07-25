// This file is @generated by prost-build.
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct SubscribePacketsRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribePacketsResponse {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<super::shared::Header>,
    #[prost(message, optional, tag = "2")]
    pub batch: ::core::option::Option<super::packet::PacketBatch>,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct SubscribeBundlesRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubscribeBundlesResponse {
    #[prost(message, repeated, tag = "1")]
    pub bundles: ::prost::alloc::vec::Vec<super::bundle::BundleUuid>,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct BlockBuilderFeeInfoRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BlockBuilderFeeInfoResponse {
    #[prost(string, tag = "1")]
    pub pubkey: ::prost::alloc::string::String,
    /// commission (0-100)
    #[prost(uint64, tag = "2")]
    pub commission: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountsOfInterest {
    /// use * for all accounts
    #[prost(string, repeated, tag = "1")]
    pub accounts: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct AccountsOfInterestRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AccountsOfInterestUpdate {
    #[prost(string, repeated, tag = "1")]
    pub accounts: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct ProgramsOfInterestRequest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProgramsOfInterestUpdate {
    #[prost(string, repeated, tag = "1")]
    pub programs: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// A series of packets with an expiration attached to them.
/// The header contains a timestamp for when this packet was generated.
/// The expiry is how long the packet batches have before they expire and are forwarded to the validator.
/// This provides a more censorship resistant method to MEV than block engines receiving packets directly.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ExpiringPacketBatch {
    #[prost(message, optional, tag = "1")]
    pub header: ::core::option::Option<super::shared::Header>,
    #[prost(message, optional, tag = "2")]
    pub batch: ::core::option::Option<super::packet::PacketBatch>,
    #[prost(uint32, tag = "3")]
    pub expiry_ms: u32,
}
/// Packets and heartbeats are sent over the same stream.
/// ExpiringPacketBatches have an expiration attached to them so the block engine can track
/// how long it has until the relayer forwards the packets to the validator.
/// Heartbeats contain a timestamp from the system and is used as a simple and naive time-sync mechanism
/// so the block engine has some idea on how far their clocks are apart.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PacketBatchUpdate {
    #[prost(oneof = "packet_batch_update::Msg", tags = "1, 2")]
    pub msg: ::core::option::Option<packet_batch_update::Msg>,
}
/// Nested message and enum types in `PacketBatchUpdate`.
pub mod packet_batch_update {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Msg {
        #[prost(message, tag = "1")]
        Batches(super::ExpiringPacketBatch),
        #[prost(message, tag = "2")]
        Heartbeat(super::super::shared::Heartbeat),
    }
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct StartExpiringPacketStreamResponse {
    #[prost(message, optional, tag = "1")]
    pub heartbeat: ::core::option::Option<super::shared::Heartbeat>,
}
/// Generated client implementations.
pub mod block_engine_validator_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value
    )]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    /// / Validators can connect to Block Engines to receive packets and bundles.
    #[derive(Debug, Clone)]
    pub struct BlockEngineValidatorClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl BlockEngineValidatorClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> BlockEngineValidatorClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> BlockEngineValidatorClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            BlockEngineValidatorClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// / Validators can subscribe to the block engine to receive a stream of packets
        pub async fn subscribe_packets(
            &mut self,
            request: impl tonic::IntoRequest<super::SubscribePacketsRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::SubscribePacketsResponse>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/block_engine.BlockEngineValidator/SubscribePackets",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "block_engine.BlockEngineValidator",
                "SubscribePackets",
            ));
            self.inner.server_streaming(req, path, codec).await
        }
        /// / Validators can subscribe to the block engine to receive a stream of simulated and profitable bundles
        pub async fn subscribe_bundles(
            &mut self,
            request: impl tonic::IntoRequest<super::SubscribeBundlesRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::SubscribeBundlesResponse>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/block_engine.BlockEngineValidator/SubscribeBundles",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "block_engine.BlockEngineValidator",
                "SubscribeBundles",
            ));
            self.inner.server_streaming(req, path, codec).await
        }
        /// Block builders can optionally collect fees. This returns fee information if a block builder wants to
        /// collect one.
        pub async fn get_block_builder_fee_info(
            &mut self,
            request: impl tonic::IntoRequest<super::BlockBuilderFeeInfoRequest>,
        ) -> std::result::Result<tonic::Response<super::BlockBuilderFeeInfoResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/block_engine.BlockEngineValidator/GetBlockBuilderFeeInfo",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "block_engine.BlockEngineValidator",
                "GetBlockBuilderFeeInfo",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated client implementations.
pub mod block_engine_relayer_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value
    )]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    /// / Relayers can forward packets to Block Engines.
    /// / Block Engines provide an AccountsOfInterest field to only send transactions that are of interest.
    #[derive(Debug, Clone)]
    pub struct BlockEngineRelayerClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl BlockEngineRelayerClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> BlockEngineRelayerClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> BlockEngineRelayerClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            BlockEngineRelayerClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// / The block engine feeds accounts of interest (AOI) updates to the relayer periodically.
        /// / For all transactions the relayer receives, it forwards transactions to the block engine which write-lock
        /// / any of the accounts in the AOI.
        pub async fn subscribe_accounts_of_interest(
            &mut self,
            request: impl tonic::IntoRequest<super::AccountsOfInterestRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::AccountsOfInterestUpdate>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/block_engine.BlockEngineRelayer/SubscribeAccountsOfInterest",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "block_engine.BlockEngineRelayer",
                "SubscribeAccountsOfInterest",
            ));
            self.inner.server_streaming(req, path, codec).await
        }
        pub async fn subscribe_programs_of_interest(
            &mut self,
            request: impl tonic::IntoRequest<super::ProgramsOfInterestRequest>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::ProgramsOfInterestUpdate>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/block_engine.BlockEngineRelayer/SubscribeProgramsOfInterest",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "block_engine.BlockEngineRelayer",
                "SubscribeProgramsOfInterest",
            ));
            self.inner.server_streaming(req, path, codec).await
        }
        /// Validators can subscribe to packets from the relayer and receive a multiplexed signal that contains a mixture
        /// of packets and heartbeats.
        /// NOTE: This is a bi-directional stream due to a bug with how Envoy handles half closed client-side streams.
        /// The issue is being tracked here: https://github.com/envoyproxy/envoy/issues/22748. In the meantime, the
        /// server will stream heartbeats to clients at some reasonable cadence.
        pub async fn start_expiring_packet_stream(
            &mut self,
            request: impl tonic::IntoStreamingRequest<Message = super::PacketBatchUpdate>,
        ) -> std::result::Result<
            tonic::Response<tonic::codec::Streaming<super::StartExpiringPacketStreamResponse>>,
            tonic::Status,
        > {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/block_engine.BlockEngineRelayer/StartExpiringPacketStream",
            );
            let mut req = request.into_streaming_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "block_engine.BlockEngineRelayer",
                "StartExpiringPacketStream",
            ));
            self.inner.streaming(req, path, codec).await
        }
    }
}
