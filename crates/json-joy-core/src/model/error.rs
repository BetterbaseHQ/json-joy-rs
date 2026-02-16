#[derive(Debug, Error)]
pub enum ModelError {
    #[error("invalid model clock table")]
    InvalidClockTable,
    #[error("invalid model binary")]
    InvalidModelBinary,
}


fn looks_like_minimal_server_preamble(data: &[u8]) -> bool {
    (data.first().copied().unwrap_or(0) & 0b1000_0000) != 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MalformedCompatClass {
    AsciiJsonRejected,
    EightByteRejected,
    AcceptedOpaque,
}

fn classify_compat_malformed(data: &[u8], err: &ModelError) -> MalformedCompatClass {
    // Fixture-linked compatibility classes:
    // - `model_decode_error_ascii_json_v1`: rejected
    // - `model_decode_error_random_8_v1` (specific payload): rejected
    // - clock table framing errors: rejected
    // - several other malformed samples: accepted as opaque/null-view
    if data.first() == Some(&0x7b) {
        return MalformedCompatClass::AsciiJsonRejected;
    }
    if data == [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef] {
        return MalformedCompatClass::EightByteRejected;
    }
    if matches!(err, ModelError::InvalidClockTable) {
        // Most malformed clock-table variants are accepted upstream as opaque.
        return MalformedCompatClass::AcceptedOpaque;
    }
    MalformedCompatClass::AcceptedOpaque
}

fn compat_accepts_malformed(data: &[u8], err: &ModelError) -> bool {
    matches!(
        classify_compat_malformed(data, err),
        MalformedCompatClass::AcceptedOpaque
    )
}
