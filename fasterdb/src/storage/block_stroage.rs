use std::{fs::File, io::{Read, Seek, SeekFrom, Write}, path::PathBuf, sync::{Arc, RwLock, RwLockWriteGuard}};

use crate::storage::{block::{Block, BLOCK_DATA_SIZE, TOTAL_BLOCK_SIZE}, serialization::{FromBytes, FromBytesError, ToBytes}};

#[derive(Clone, Debug)]
pub enum StorageOption {
    File(PathBuf),
}

pub trait WriteSeek: Write + Seek {}
impl<T: Write + Seek> WriteSeek for T {}

pub trait ReadSeek: Read + Seek {}
impl<T: Read + Seek> ReadSeek for T {}

#[derive(Clone)]
pub enum BlockSeek{
    Start(u64),
    Current(i64),
}

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
    FromBytesError(FromBytesError),
    FromReaderError(String)
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

type WriterGuard<'a> = RwLockWriteGuard<'a, Box<dyn WriteSeek>>;
type ReaderGuard<'a> = RwLockWriteGuard<'a, Box<(dyn ReadSeek + 'static)>>;

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

    fn get_seek(&self, seek: BlockSeek) -> Option<SeekFrom>{
        match seek {
            BlockSeek::Current(pos) => Some(SeekFrom::Current((pos - 1) * TOTAL_BLOCK_SIZE as i64)),
            BlockSeek::Start(pos) => Some(SeekFrom::Start(self.header_size + (pos * TOTAL_BLOCK_SIZE as u64))),
        }
    }

    fn get_writer(&self) -> Result<WriterGuard, WriterError> {
        self.fd.write().map_err(|e| {
            WriterError::LockError(format!("Failed to acquire write lock {:?}", e))
        })
    }

    pub fn write(&self, block: Block, seek: BlockSeek) -> Result<usize, WriterError>{
        let buf = block.to_bytes_vec();
        let mut writer = self.get_writer()?;

        writer.seek(self.get_seek(seek).unwrap())?;
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

    fn get_seek(&self, seek: BlockSeek) -> SeekFrom{
        match seek {
            BlockSeek::Current(pos) => SeekFrom::Current((pos - 1) * TOTAL_BLOCK_SIZE as i64),
            BlockSeek::Start(pos) => SeekFrom::Start(self.header_size + (pos * TOTAL_BLOCK_SIZE as u64)),
        }
    }

    fn get_reader(&self) -> Result<ReaderGuard, ReaderError> {
        self.fd.write().map_err(|e| {
            ReaderError::LockError(format!("Failed to acquire write lock {:?}", e))
        })
    }

    pub fn read_block(&self, position: BlockSeek) -> Result<Block, ReaderError>{
        let mut reader = self.get_reader()?;
        let mut buf = [0u8;TOTAL_BLOCK_SIZE];

        reader.seek(self.get_seek(position))?;
        
        reader.read_exact(&mut buf)?;
        Ok(Block::from_bytes_vec(&buf)?)
    }

    pub fn read_full_item(&self, position: BlockSeek) -> Result<Vec<u8>, ReaderError>{
        let mut out: Vec<u8> = vec![];
        let mut search_pos = Some(position.clone());
        while let Some(new_pos) = search_pos {
            let block = self.read_block(new_pos.clone())?;
            out.append(&mut block.get_data(BLOCK_DATA_SIZE, 0).map_err(|e| {
                ReaderError::FromReaderError(e)
            } )?);
            search_pos = block.get_next_offset();
        }
        Ok(out)

    }
}

#[cfg(test)]
mod tests {
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
        let mut out_vec = vec![0u8;2016];
        out_vec[10] = 10;
        out_vec[11] = 20;
        out_vec[12] = 30;

        out_vec[1034] = 10;
        out_vec[1035] = 20;
        out_vec[1036] = 30;

        out_vec
    }
 
    #[test]
    fn test_writer_and_reader() {
        let block = gen_block(true);
        
        let tmpfile = tempfile::NamedTempFile::new().unwrap();
        let path = tmpfile.path().to_path_buf();
        let writer_res = Writer::new(StorageOption::File(path.clone()));
        assert!(writer_res.is_ok());
        
        let writer = writer_res.unwrap();
        let result = writer.write(block, BlockSeek::Start(0)).unwrap();
        assert_eq!(result, 1024);
        
        let block = gen_block(false);
        let result = writer.write(block, BlockSeek::Current(1)).unwrap(); // right to the next block
        assert_eq!(result, 1024);

        let flush_res = writer.flush();
        assert!(flush_res.is_ok());

        // Now test the reader

        let reader_res = Reader::new(StorageOption::File(path));
        assert!(reader_res.is_ok());
        let reader = reader_res.unwrap();
        let data_res = reader.read_full_item(BlockSeek::Start(0));
        assert!(data_res.is_ok());
        let data = data_res.unwrap();
        assert_eq!(data[0], get_file_expected_data()[0])

    }

}