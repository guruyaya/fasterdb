use std::{fs::File, io::{Seek, SeekFrom, Write}, path::PathBuf, sync::{Arc, RwLock}};

use crate::storage::{block::{Block, TOTAL_BLOCK_SIZE}, serialization::ToBytes};

#[derive(Clone, Debug)]
pub enum StorageOption {
    File(PathBuf),
    Memory,
}

trait WriteSeek: Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}

pub struct Writer{
    stored_in: StorageOption,
    fd: Arc<RwLock<Box<dyn WriteSeek>>>,
    header_size: u64
}

#[derive(Debug)]
pub enum WriterError {
    Io(std::io::Error),
    LockError(String), // משתמשים ב-String במקום PoisonError כדי להימנע מבעיות גנריות
}

impl From<std::io::Error> for WriterError{
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl Writer{
    pub fn new(stored_in: StorageOption) -> Result<Self, WriterError> {
        Ok(Self{
            header_size: 0,
            stored_in: stored_in.clone(),
            fd : match stored_in {
                StorageOption::File(path) => {
                    let file = File::create(path)?;
                    Arc::new(RwLock::new(Box::new(file) as Box<dyn WriteSeek>))
                }
                StorageOption::Memory => {
                    let buffer = Vec::new();
                    let cursor = std::io::Cursor::new(buffer);
                    Arc::new(RwLock::new(Box::new(cursor) as Box<dyn WriteSeek>))
                }
            }
        })
    }
    pub fn write(&self, block: Block, position: u64) -> Result<usize, WriterError>{
        let buf = block.to_bytes_vec();
        let mut writer = self.fd.write().map_err(|e| {
            WriterError::LockError(format!("Failed to acquire write lock {:?}", e))
        })?;
        writer.seek(SeekFrom::Start(self.header_size + (position * TOTAL_BLOCK_SIZE as u64)))?; // 1024 is the size
        let written_len = writer.write(&buf)?;
        Ok(written_len)
    }

    pub fn flush(&self) -> Result<(), WriterError> {
        let mut writer = self.fd.write().map_err(|e| {
            WriterError::LockError(format!("Failed to acquire write lock for flush: {:?}", e))
        })?;
        writer.flush().map_err(WriterError::Io)
    }

}
#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use crate::storage::block::Block;
    use tempfile;

    fn gen_block() -> Block{
        let mut block = Block{
            id: 10,
            data: [0u8;1008],
            next_block_offset: 10
        };
        block.data[10] = 10;
        block.data[11] = 20;
        block.data[12] = 30;
        block
    }

    fn get_block_expected_data() -> Vec<u8> {
        let mut out_vec = vec![0u8;1024];
        out_vec[0] = 10;
        out_vec[18] = 10;
        out_vec[19] = 20;
        out_vec[20] = 30;
        out_vec[1016] = 10;

        out_vec
    }
    #[test]
    fn test_writer_file_creation_and_block_insertion() {
        let block = gen_block();

        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_path_buf();
        let writer_res = Writer::new(StorageOption::File(path));
        assert!(writer_res.is_ok());

        let writer = writer_res.unwrap();
        let result = writer.write(block, 0).unwrap();
        assert_eq!(result, 1024);
        let flush_res = writer.flush();
        assert!(flush_res.is_ok());

        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        let mut buffer = vec![0u8; 1024];
        let read_res = tmpfile.read_exact(&mut buffer);
        assert!(read_res.is_ok());
        assert_eq!(buffer, get_block_expected_data());
    }

}