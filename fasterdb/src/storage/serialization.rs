use std::{io::{Read, Write}, string::FromUtf8Error};
// Helper functions for binary serialization

pub enum SizeExtraction {
    Constant(usize),
    FromStart
}
fn add_vectors_collect<T: Clone>(vec1: Vec<T>, vec2: Vec<T>) -> Vec<T> {
    vec1.into_iter().chain(vec2.into_iter()).collect()
}

#[allow(dead_code)]
pub trait ToBytes {
    fn to_bytes_vec(&self) -> Vec<u8>;
}

impl ToBytes for u64 {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ToBytes for u32 {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }   
}

impl ToBytes for i64 {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}


impl ToBytes for i32 {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ToBytes for usize {
    fn to_bytes_vec(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl<T: ToBytes> ToBytes for Vec<T> {
    fn to_bytes_vec(&self) -> Vec<u8> {
        let vec_len = self.len();
        let out_vec: Vec<u8> = vec_len.to_bytes_vec();
        let vec_to_u8: Vec<u8> = self.iter().map(|item| item.to_bytes_vec()).flatten().collect();
        add_vectors_collect(out_vec, vec_to_u8)
    }
}

impl ToBytes for String {
    fn to_bytes_vec(&self) -> Vec<u8> {
        let str_len = self.len();
        let out_vec: Vec<u8> = str_len.to_bytes_vec();
        let str_vec = Vec::from(self.as_bytes());

        add_vectors_collect(out_vec, str_vec)
    } 
}


#[allow(unused)]
#[derive(Debug)]
pub enum FromBytesError {
    Utf8Error(std::string::FromUtf8Error),
    Io(std::io::Error),
    ReadLenError,
}

impl From<std::io::Error> for FromBytesError {
    fn from(err: std::io::Error) -> Self {
        FromBytesError::Io(err)
    }
}

impl From<FromUtf8Error> for FromBytesError {
    fn from(err: FromUtf8Error) -> Self {
        FromBytesError::Utf8Error(err)
    }
}

// read trait
#[allow(dead_code)]
pub trait FromBytes {
    fn from_bytes_vec(bytes: &[u8]) -> Result<Self, FromBytesError> where Self:Sized;
    fn get_size_strategy() -> SizeExtraction;
    fn get_read_size(reader: &mut dyn Read) -> Result<usize, FromBytesError>{
        match Self::get_size_strategy() {
            SizeExtraction::Constant(size) => Ok(size),
            SizeExtraction::FromStart => {
                let mut buffer = vec![0u8; std::mem::size_of::<usize>()];
                reader.read_exact(&mut buffer)?;
                Ok(usize::from_bytes_vec(&buffer)?)
            }
        }
    }
    fn read(reader: &mut dyn Read) -> Result<Self, FromBytesError> where Self:Sized {
        match Self::get_size_strategy() {
            SizeExtraction::Constant(size) => {
                let mut buffer = vec![0u8; size];
                reader.read_exact(&mut buffer)?;
                Ok(Self::from_bytes_vec(&buffer)?)
            },
            SizeExtraction::FromStart => {
                let len = Self::get_read_size(reader)?;
                let mut buffer = vec![0u8; len];
                reader.read_exact(&mut buffer)?;
                
                Ok(Self::from_bytes_vec(&buffer)?)
            }
        }
    }
}

impl FromBytes for usize {
    fn from_bytes_vec(bytes: &[u8]) -> Result<Self, FromBytesError> {
        let mut arr = [0u8; std::mem::size_of::<Self>()];
        arr.copy_from_slice(bytes);
        Ok(usize::from_le_bytes(arr))
    }
    fn get_size_strategy() -> SizeExtraction {
        SizeExtraction::Constant(std::mem::size_of::<Self>())
    }
}

impl FromBytes for u64 {
    fn from_bytes_vec(bytes: &[u8]) -> Result<Self, FromBytesError> {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(u64::from_le_bytes(arr))
    }
    fn get_size_strategy() -> SizeExtraction {
        SizeExtraction::Constant(std::mem::size_of::<Self>())
    }
}


impl FromBytes for i64 {
    fn from_bytes_vec(bytes: &[u8]) -> Result<Self, FromBytesError> {
        let mut arr = [0u8; 8];
        arr.copy_from_slice(bytes);
        Ok(i64::from_le_bytes(arr))
    }
    fn get_size_strategy() -> SizeExtraction {
        SizeExtraction::Constant(std::mem::size_of::<Self>())
    }
}

impl FromBytes for u32 {
    fn from_bytes_vec(bytes: &[u8]) -> Result<Self, FromBytesError> {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);
        Ok(u32::from_le_bytes(arr))
    }   
    fn get_size_strategy() -> SizeExtraction {
        SizeExtraction::Constant(std::mem::size_of::<Self>())
    }
}


impl FromBytes for i32 {
    fn from_bytes_vec(bytes: &[u8]) -> Result<Self, FromBytesError> {
        let mut arr = [0u8; 4];
        arr.copy_from_slice(bytes);
        Ok(i32::from_le_bytes(arr))
    }   
    fn get_size_strategy() -> SizeExtraction {
        SizeExtraction::Constant(std::mem::size_of::<Self>())
    }
}


impl FromBytes for String {
    fn from_bytes_vec(bytes: &[u8]) -> Result<Self, FromBytesError> {
        let vec = bytes.to_vec();
        Ok(String::from_utf8(vec)?)
    }   
    fn get_size_strategy() -> SizeExtraction {
        SizeExtraction::FromStart
    }
}

/// Write any value as little-endian bytes
pub fn write_bytes(writer: &mut dyn Write, value: impl ToBytes) -> Result<(), std::io::Error> {
    writer.write_all(&value.to_bytes_vec())
}


/// Read any value as little-endian bytes
pub fn read_bytes<T: FromBytes>(reader: &mut dyn Read) -> Result<T, FromBytesError> {
    T::read(reader)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    #[test]
    fn test_io_u64() {
        let value: u64 = 15;
        let mut buffer = File::create("data/foou64.dat").unwrap();
        let res = write_bytes(&mut buffer, value);
        assert_eq!(res.unwrap(), ());

        let mut buffer= File::open("data/foou64.dat").unwrap();
        let new_res: Result<u64, FromBytesError> = read_bytes(&mut buffer);
        assert_eq!(new_res.unwrap(), value);
    }

    
    #[test]
    fn test_io_i64() {
        let value: i64 = -19;
        let mut buffer = File::create("data/fooi64.dat").unwrap();
        let res = write_bytes(&mut buffer, value);
        assert_eq!(res.unwrap(), ());

        let mut buffer= File::open("data/fooi64.dat").unwrap();
        let new_res: Result<i64, FromBytesError> = read_bytes(&mut buffer);
        assert_eq!(new_res.unwrap(), value);
    }

    
    #[test]
    fn test_io_u32() {
        let value: u32 = 30;
        let mut buffer = File::create("data/foou32.dat").unwrap();
        let res = write_bytes(&mut buffer, value);
        assert_eq!(res.unwrap(), ());

        let mut buffer= File::open("data/foou32.dat").unwrap();
        let new_res: Result<u32, FromBytesError> = read_bytes(&mut buffer);
        assert_eq!(new_res.unwrap(), value);
    }

    
    #[test]
    fn test_io_i32() {
        let value: i32 = -12;
        let mut buffer = File::create("data/fooi32.dat").unwrap();
        let res = write_bytes(&mut buffer, value);
        assert_eq!(res.unwrap(), ());

        let mut buffer= File::open("data/fooi32.dat").unwrap();
        let new_res: Result<i32, FromBytesError> = read_bytes(&mut buffer);
        assert_eq!(new_res.unwrap(), value);
    }

    #[test]
    fn test_io_string() {
        let value = String::from("יאיר אשל דובר עברית");
        let value_clone = value.clone();

        let mut buffer = File::create("data/foostring.dat").unwrap();
        let res = write_bytes(&mut buffer, value);
        assert_eq!(res.unwrap(), ());

        let mut buffer= File::open("data/foostring.dat").unwrap();
        let new_res: Result<String, FromBytesError> = read_bytes(&mut buffer);
        assert_eq!(new_res.unwrap(), value_clone);

    }

    #[test]
    fn test_io_vec() {
        let value:Vec<u64> = vec![10, 20, 30];
        // let value_clone = value.clone();

        let mut buffer = File::create("data/foovec46.dat").unwrap();
        let res = write_bytes(&mut buffer, value);
        assert_eq!(res.unwrap(), ());

        let value:Vec<String> = vec!["I am the smart".to_string(), "I am stupid".to_string()];
        // let value_clone = value.clone();

        let mut buffer = File::create("data/foovecstrings.dat").unwrap();
        let res = write_bytes(&mut buffer, value);
        assert_eq!(res.unwrap(), ());

        
    }

}