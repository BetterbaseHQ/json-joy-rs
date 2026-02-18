//! WebSocket frame opcodes (RFC 6455 §5.2).
//!
//! Upstream reference: `json-pack/src/ws/constants.ts`

/// WebSocket frame opcode values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WsFrameOpcode {
    /// Continuation fragment of a data frame.
    Continue = 0,
    /// UTF-8 text data frame.
    Text = 1,
    /// Binary data frame.
    Binary = 2,
    /// Close control frame.
    Close = 8,
    /// Ping control frame.
    Ping = 9,
    /// Pong control frame.
    Pong = 10,
}

impl WsFrameOpcode {
    /// Minimum opcode value for control frames.
    pub const MIN_CONTROL_OPCODE: u8 = 8;
}

impl TryFrom<u8> for WsFrameOpcode {
    type Error = u8;
    fn try_from(v: u8) -> Result<Self, u8> {
        match v {
            0 => Ok(Self::Continue),
            1 => Ok(Self::Text),
            2 => Ok(Self::Binary),
            8 => Ok(Self::Close),
            9 => Ok(Self::Ping),
            10 => Ok(Self::Pong),
            other => Err(other),
        }
    }
}
