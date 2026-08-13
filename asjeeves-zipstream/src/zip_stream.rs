use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

use flate2::{Compress, Compression, FlushCompress};

use crate::directory_metadata::DirectoryMetadata;
use crate::file_metadata::FileMetadata;

pub struct ZipStream<W> {
    current_offset: u32,
    directory_metadata: DirectoryMetadata,
    writer: W,
}

impl<W> ZipStream<W>
where
    W: AsyncWrite + Unpin + AsyncWriteExt,
{
    pub fn new(writer: W) -> Self {
        Self {
            current_offset: 0,
            directory_metadata: DirectoryMetadata::default(),
            writer,
        }
    }

    pub async fn add_file<R>(
        &mut self,
        name: impl Into<Box<str>>,
        reader: &mut R,
    ) -> Result<(), std::io::Error>
    where
        R: AsyncRead + Unpin,
    {
        let mut compressor = Compress::new(Compression::default(), false);

        // checksum.
        let mut hasher = crc32fast::Hasher::new();

        // 4k buffers
        let mut comp_buf = [0u8; 4096];
        let mut read_buf = [0u8; 4096];

        let mut meta = FileMetadata {
            name: name.into(),
            local_header_offset: self.current_offset,
            ..FileMetadata::default()
        };

        let header: Box<[u8]> = meta.as_file_header()?;

        self.writer.write_all(&header).await?;

        self.current_offset += header.len() as u32;

        // compress and stream file content
        loop {
            let bytes_read: usize = reader.read(&mut read_buf).await?;

            if bytes_read == 0 {
                break; // EOF
            }

            let chunk: &[u8] = &read_buf[..bytes_read];

            meta.uncompressed_size += bytes_read as u32;
            hasher.update(chunk);

            // compress the 64k chunk in 4k chunks
            let mut input_pos = 0;

            while input_pos < chunk.len() {
                let before_in = compressor.total_in();
                let before_out = compressor.total_out();

                compressor
                    .compress(&chunk[input_pos..], &mut comp_buf, FlushCompress::None)
                    .map_err(std::io::Error::other)?;

                let consumed = (compressor.total_in() - before_in) as usize;
                let produced = (compressor.total_out() - before_out) as usize;

                if produced > 0 {
                    meta.compressed_size += produced as u32;
                    self.current_offset += produced as u32;

                    self.writer.write_all(&comp_buf[..produced]).await?;
                }

                input_pos += consumed;
            }
        }

        // Finish deflate stream
        loop {
            let before_out = compressor.total_out();

            compressor
                .compress(&[], &mut comp_buf, FlushCompress::Finish)
                .map_err(std::io::Error::other)?;

            let produced = (compressor.total_out() - before_out) as usize;

            if produced > 0 {
                meta.compressed_size += produced as u32;
                self.current_offset += produced as u32;

                self.writer.write_all(&comp_buf[..produced]).await?;
            } else {
                break;
            }
        }

        meta.checksum = hasher.finalize();

        // data descriptor
        let dd: Box<[u8]> = meta.as_data_descriptor()?;

        self.writer.write_all(&dd).await?;
        self.current_offset += dd.len() as u32;

        self.directory_metadata.files.push(meta);

        Ok(())
    }

    pub async fn finish(mut self) -> Result<(), std::io::Error> {
        self.directory_metadata.offset = self.current_offset;

        // Central directory
        for file in &self.directory_metadata.files {
            let entry: Box<[u8]> = file.as_dir_entry()?;
            self.writer.write_all(&entry).await?;
            self.directory_metadata.total_size += entry.len() as u32;
        }

        // End of Central Directory
        let eocd: Box<[u8]> = self.directory_metadata.as_directory_footer()?;

        self.writer.write_all(&eocd).await?;

        Ok(())
    }
}
