use std::{fs::File, io::{Read, Seek, SeekFrom, Write}, path::PathBuf, sync::{Arc, RwLock, RwLockWriteGuard}};

use crate::storage::{block::{Block, TOTAL_BLOCK_SIZE}, serialization::{FromBytes, FromBytesError, ToBytes}};

#[derive(Clone, Debug)]
pub enum StorageOption {
    File(PathBuf),
}

pub trait WriteSeek: Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

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

#[derive(Debug)]
pub enum ReaderError {
    Io(std::io::Error),
    LockError(String), // משתמשים ב-String במקום PoisonError כדי להימנע מבעיות גנריות
    FromBytesError(FromBytesError)
}

impl From<std::io::Error> for WriterError{
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<std::io::Error> for ReaderError{
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<FromBytesError> for ReaderError {
    fn from(err: FromBytesError) -> Self {
        Self::FromBytesError(err)
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
            }
        })
    }

    fn get_writer(&self) -> Result<RwLockWriteGuard<Box<(dyn WriteSeek + 'static)>>, WriterError> {
        self.fd.write().map_err(|e| {
            WriterError::LockError(format!("Failed to acquire write lock {:?}", e))
        })
    }

    pub fn write(&self, block: Block, position: u64) -> Result<usize, WriterError>{
        let buf = block.to_bytes_vec();
        let mut writer = self.get_writer()?;
        
        writer.seek(SeekFrom::Start(self.header_size + (position * TOTAL_BLOCK_SIZE as u64)))?; // 1024 is the size
        let written_len = writer.write(&buf)?;
        Ok(written_len)
    }

    pub fn write_seek(&self, block: Block, seek: i64) -> Result<usize, WriterError>{
        let buf = block.to_bytes_vec();
        let mut writer = self.get_writer()?;

        // writer.seek(SeekFrom::Current((seek - 1) * TOTAL_BLOCK_SIZE as i64))?; 
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

pub struct Reader{
    stored_in: StorageOption,
    fd: Arc<RwLock<Box<dyn ReadSeek>>>,
    header_size: u64
}

impl Reader {
    pub fn new(stored_in: StorageOption) -> Result<Self, ReaderError> {
        Ok(Self{
            header_size: 0,
            stored_in: stored_in.clone(),
            fd : match stored_in {
                StorageOption::File(path) => {
                    let file = File::open(path)?;
                    Arc::new(RwLock::new(Box::new(file) as Box<dyn ReadSeek>))
                }
            }
        })
    }

    pub fn read_block(&self, position: u64) -> Result<Block, ReaderError>{
        let mut reader = self.fd.write().map_err(|e| {
            ReaderError::LockError(format!("Failed to acquire read lock for flush: {:?}", e))
        })?;
        let mut buf = [0u8;TOTAL_BLOCK_SIZE];
        let seek_position = self.header_size + (position * TOTAL_BLOCK_SIZE as u64);
        reader.seek(SeekFrom::Start(seek_position))?;

        reader.read_exact(&mut buf)?;
        Ok(Block::from_bytes_vec(&buf)?)

    }
}
#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use crate::storage::block::Block;
    use tempfile;

    fn gen_block(first: bool) -> Block{
        let mut block = Block{
            id: 10,
            data: [0u8;1008],
            next_block_offset: {if first {1} else {0}}
        };
        block.data[10] = 10;
        block.data[11] = 20;
        block.data[12] = 30;
        block
    }

    fn get_file_expected_data() -> Vec<u8> {
        let mut out_vec = vec![0u8;2048];
        out_vec[0] = 10;
        out_vec[18] = 10;
        out_vec[19] = 20;
        out_vec[20] = 30;
        out_vec[1016] = 1;

        out_vec[1024] = 10;
        out_vec[1042] = 10;
        out_vec[1043] = 20;
        out_vec[1044] = 30;

        out_vec


    }
    #[test]
    fn test_writer_and_reader() {
        let block = gen_block(true);
        
        let mut tmpfile = tempfile::NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_path_buf();
        let writer_res = Writer::new(StorageOption::File(path));
        assert!(writer_res.is_ok());
        
        let writer = writer_res.unwrap();
        let result = writer.write(block, 0).unwrap();
        assert_eq!(result, 1024);
        
        let block = gen_block(false);
        let result = writer.write_seek(block, 1).unwrap(); // right to the next block
        assert_eq!(result, 1024);

        let flush_res = writer.flush();
        assert!(flush_res.is_ok());

        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        let mut buffer = vec![0u8; 2048];
        let read_res = tmpfile.read_exact(&mut buffer);
        assert!(read_res.is_ok());
        assert_eq!(buffer, get_file_expected_data());
        assert_eq!(buffer, get_file_expected_data());
    


    }

}