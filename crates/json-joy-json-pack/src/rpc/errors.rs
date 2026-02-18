//! ONC RPC error types.
//!
//! Upstream reference: `json-pack/src/rpc/errors.ts`

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RpcDecodingError {
    #[error("RPC_DECODING: {0}")]
    InvalidMessage(String),
    #[error("RPC_DECODING")]
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum RpcEncodingError {
    #[error("RPC_ENCODING: {0}")]
    InvalidMessage(String),
    #[error("RPC_ENCODING")]
    Generic,
}
