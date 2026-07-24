use std::io::Write;

use crate::file_metadata::FileMetadata;

#[derive(Debug, Default)]
pub struct DirectoryMetadata {
    pub files: Vec<FileMetadata>,
    pub offset: u32,
    pub total_size: u32,
}

impl DirectoryMetadata {
    pub fn as_directory_footer(&self) -> Result<Box<[u8]>, std::io::Error> {
        let num_files: [u8; 2] = (self.files.len() as u16).to_le_bytes();

        let mut eocd = Vec::with_capacity(22);
        eocd.write_all(&[0x50, 0x4b, 0x05, 0x06])?;
        eocd.write_all(&[0x00, 0x00])?;
        eocd.write_all(&[0x00, 0x00])?;
        eocd.write_all(&num_files)?;
        eocd.write_all(&num_files)?;
        eocd.write_all(&self.total_size.to_le_bytes())?;
        eocd.write_all(&self.offset.to_le_bytes())?;
        eocd.write_all(&[0x00, 0x00])?;

        Ok(eocd.into_boxed_slice())
    }
}
