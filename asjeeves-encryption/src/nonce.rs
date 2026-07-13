use uuid::Uuid;

pub fn generate_nonce() -> Box<str> {
    let nonce = Uuid::now_v7();

    let nonce = nonce.to_string();

    nonce.into_boxed_str()
}
