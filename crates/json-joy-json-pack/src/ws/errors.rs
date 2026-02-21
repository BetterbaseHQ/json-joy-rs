//! WebSocket codec error types.
//!
//! Upstream reference: `json-pack/src/ws/errors.ts`

/// Error type for WebSocket frame encoding failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WsFrameEncodingError {
    #[error("WS_FRAME_ENCODING")]
    InvalidFrame,
}
