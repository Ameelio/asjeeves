mod directory_metadata;
mod file_metadata;
mod zip_stream;

pub use zip_stream::ZipStream;

#[cfg(test)]
mod test {
    use super::ZipStream;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn it_should_generate_a_valid_zip() {
        {
            let writer = File::create("/tmp/output.zip").await.unwrap();
            let mut reader = File::open(concat!(env!("CARGO_MANIFEST_DIR"), "/../LICENSE.md"))
                .await
                .unwrap();

            let mut zipstream = ZipStream::new(writer);

            zipstream.add_file("LICENSE.md", &mut reader).await.unwrap();

            zipstream.finish().await.unwrap();
        }

        let mut reader = File::open("/tmp/output.zip").await.unwrap();

        let mut contents: Vec<u8> = Vec::new();

        reader.read_to_end(&mut contents).await.unwrap();

        let mut hasher = crc32fast::Hasher::new();

        hasher.update(&contents);

        let checksum: u32 = hasher.finalize();

        assert_eq!(3025795208, checksum);

        tokio::fs::remove_file("/tmp/output.zip").await.unwrap();
    }
}
