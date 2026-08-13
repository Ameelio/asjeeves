use uuid::Uuid;

pub fn generate_nonce() -> Box<str> {
    let nonce = Uuid::now_v7();

    let nonce = nonce.to_string();

    nonce.into_boxed_str()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_should_generate_uniq_values() {
        let n1 = generate_nonce();
        let n2 = generate_nonce();
        let n3 = generate_nonce();

        assert_ne!(n1, n2);
        assert_ne!(n1, n3);
        assert_ne!(n2, n3);
    }
}
