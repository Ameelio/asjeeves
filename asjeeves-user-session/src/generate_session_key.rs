/// Generates a session key - isolated to ensure we have an agreement between reading and writing
/// sessions.
pub fn generate_session_key(prefix: &str, session_id: &str) -> String {
    format!("{}:{}", prefix, session_id)
}

