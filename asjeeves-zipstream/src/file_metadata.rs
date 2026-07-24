use std::io::Write;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FileMetadata {
    pub name: Box<str>,
    pub checksum: u32,
    pub compressed_size: u32,
    pub local_header_offset: u32,
    pub uncompressed_size: u32,
}

impl FileMetadata {
    pub fn as_data_descriptor(&self) -> Result<Box<[u8]>, std::io::Error> {
        let mut dd: Vec<u8> = Vec::with_capacity(16);

        dd.write_all(&[0x50, 0x4b, 0x07, 0x08])?;
        dd.write_all(&self.checksum.to_le_bytes())?;
        dd.write_all(&self.compressed_size.to_le_bytes())?;
        dd.write_all(&self.uncompressed_size.to_le_bytes())?;

        Ok(dd.into_boxed_slice())
    }

    pub fn as_dir_entry(&self) -> Result<Box<[u8]>, std::io::Error> {
        let name_bytes: &[u8] = self.name.as_bytes();
        let mut cd: Vec<u8> = Vec::with_capacity(50 + name_bytes.len());

        cd.write_all(&[0x50, 0x4b, 0x01, 0x02])?;
        cd.write_all(&[0x14, 0x00])?;
        cd.write_all(&[0x14, 0x00])?;
        cd.write_all(&[0x08, 0x00])?;
        cd.write_all(&[0x08, 0x00])?;
        cd.write_all(&[0x00, 0x00, 0x00, 0x00])?;
        cd.write_all(&self.checksum.to_le_bytes())?;
        cd.write_all(&self.compressed_size.to_le_bytes())?;
        cd.write_all(&self.uncompressed_size.to_le_bytes())?;
        cd.write_all(&(name_bytes.len() as u16).to_le_bytes())?;
        cd.write_all(&[0x00, 0x00])?;
        cd.write_all(&[0x00, 0x00])?;
        cd.write_all(&[0x00, 0x00])?;
        cd.write_all(&[0x00, 0x00])?;
        cd.write_all(&[0x00, 0x00, 0x00, 0x00])?;
        cd.write_all(&self.local_header_offset.to_le_bytes())?;
        cd.write_all(name_bytes)?;

        Ok(cd.into_boxed_slice())
    }

    pub fn as_file_header(&self) -> Result<Box<[u8]>, std::io::Error> {
        let name_bytes: &[u8] = self.name.as_bytes();

        let len: [u8; 2] = (name_bytes.len() as u16).to_le_bytes();

        let mut bytes: Vec<u8> = Vec::with_capacity(24 + name_bytes.len());

        bytes.write_all(&[0x50, 0x4b, 0x03, 0x04])?;
        bytes.write_all(&[0x14, 0x00])?;
        bytes.write_all(&[0x08, 0x00])?;
        bytes.write_all(&[0x08, 0x00])?;
        bytes.write_all(&[0x00, 0x00, 0x00, 0x00])?;
        bytes.write_all(&[0x00, 0x00, 0x00, 0x00])?;
        bytes.write_all(&[0x00, 0x00, 0x00, 0x00])?;
        bytes.write_all(&[0x00, 0x00, 0x00, 0x00])?;
        bytes.write_all(&len)?;
        bytes.write_all(&[0x00, 0x00])?;
        bytes.write_all(name_bytes)?;

        Ok(bytes.into_boxed_slice())
    }
}
