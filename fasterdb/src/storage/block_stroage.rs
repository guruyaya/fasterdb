use std::{fs::File, io::{Seek, SeekFrom, Write}, sync::{Arc, RwLock}};

use crate::storage::{block::{Block, TOTAL_BLOCK_SIZE}, serialization::ToBytes};

#[derive(Clone, Debug)]
pub enum StorageOption {
    File(String),
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
                StorageOption::File(filename) => {
                    let file = File::create(&filename)?;
                    Arc::new(RwLock::new(Box::new(file)))
                }
                StorageOption::Memory => {
                    let buffer = Vec::new();
                    let cursor = std::io::Cursor::new(buffer);
                    Arc::new(RwLock::new(Box::new(cursor)))
                }
            }
        })
    }
    pub fn write(&self, block: Block, position: u64) -> Result<usize, WriterError>{
        let mut buf = block.to_bytes_vec();
        let mut writer = self.fd.write().map_err(|e| {
            WriterError::LockError(format!("Failed to acuire write lock {:?}", e))
        })?;
        writer.seek(SeekFrom::Start(self.header_size + (position * TOTAL_BLOCK_SIZE as u64)))?; // 1024 is the size
        let written_len = writer.write(&buf)?;
        Ok(written_len)
    }
}
    