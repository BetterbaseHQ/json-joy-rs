pub fn version() -> String {
    json_joy_core::version().to_owned()
}

pub fn generate_session_id() -> u64 {
    json_joy_core::generate_session_id()
}

pub fn is_valid_session_id(sid: u64) -> bool {
    json_joy_core::is_valid_session_id(sid)
}

uniffi::include_scaffolding!("json_joy");
