pub struct Model {
    /// Preserve the exact model wire payload for deterministic round-trips.
    bytes: Vec<u8>,
    view: Value,
}

impl Model {
    pub fn from_binary(data: &[u8]) -> Result<Self, ModelError> {
        if data.is_empty() || (data.len() < 4 && !looks_like_minimal_server_preamble(data)) {
            return Err(ModelError::InvalidClockTable);
        }

        match decode_model_view(data) {
            Ok(view) => Ok(Self {
                bytes: data.to_vec(),
                view,
            }),
            Err(err) => {
                if compat_accepts_malformed(data, &err) {
                    // Compatibility mode: keep upstream parity by accepting
                    // specific malformed classes as opaque payloads.
                    return Ok(Self {
                        bytes: data.to_vec(),
                        view: Value::Null,
                    });
                }
                Err(err)
            }
        }
    }

    pub fn to_binary(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    pub fn view(&self) -> &Value {
        &self.view
    }
}
