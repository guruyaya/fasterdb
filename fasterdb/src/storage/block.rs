
// Block structure for file storage
#[derive(Debug, Clone)]
pub struct Block {
    data: [u8; 1016],
    next_block_offset: u64,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            data: [0u8; 1016],
            next_block_offset: 0, // 0 means no next block
        }        
    }
}

impl Block {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn set_data(&mut self, data: &[u8], offset: usize) -> Result<(), String> {
        if offset + data.len() > 1016 {
            let data_len = data.len();
            let total_size = offset + data.len();
            return Err(format!("Data size {data_len} + offset {offset} = {total_size} exceeds block capacity (1016)"));
        }
        
        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    
    pub fn get_data(&self, size: usize, offset: usize) -> Result<Vec<u8>, String> {
        if size + offset > 1016 {
            return Err("Data requested, exeeds 1016".to_string());
        }
        Ok(self.data[offset..size + offset].to_vec())
    }
    
    pub fn set_next_block(&mut self, offset: u64) {
        self.next_block_offset = offset;
    }
    
    pub fn get_next_block(&self) -> u64 {
        self.next_block_offset
    }
}

#[cfg(test)]
mod test{
    use crate::storage::serialization::ToBytes;

    use super::*;

    #[test]
    fn test_new_block() {
        let block = Block::new();
        assert!(block.data.iter().fold(true, |v1, v2| -> bool {v1 && (v2 == &0)}), "Not all values are 0");
        assert_eq!(block.next_block_offset, 0);
    }

    #[test]
    fn test_block_io_values() {
        let mut block = Block::new();
        let data = String::from("I am").to_bytes_vec();
        let out_res = block.set_data(&data, 10);

        assert_eq!(out_res, Ok(()));

        let out_res = block.get_data(80, 3);
        let result = out_res.unwrap();
        assert_eq!(result.len(), 80);
        assert!(&result[0..7].iter().fold(true, |v1, v2| -> bool {v1 && (v2 == &0)}), "Not all values are 0");
        assert_eq!(&result[7..20], vec![4, 0, 0, 0, 0, 0, 0, 0, 73, 32, 97, 109, 0]);
        assert!(&result[20..].iter().fold(true, |v1, v2| -> bool {v1 && (v2 == &0)}), "Not all values are 0");

    }

    #[test]
    fn test_set_get_next_block() {
        let mut block = Block::new();
        block.set_next_block(11);

        assert_eq!(block.next_block_offset, 11);
        assert_eq!(block.get_next_block(), 11);
    }

    #[test]
    fn test_data_store_error() {
        let mut block = Block::new();
        let res = block.set_data(&[11;1016], 0);
        assert_eq!(res, Ok(()));

        let mut block = Block::new();
        let res = block.set_data(&[11], 1017);
        assert_eq!(res, Err("Data size 1 + offset 1017 = 1018 exceeds block capacity (1016)".to_string()));

        let mut block = Block::new();
        let res = block.set_data(&[11;1017], 0);
        assert_eq!(res, Err("Data size 1017 + offset 0 = 1017 exceeds block capacity (1016)".to_string()));
        
        let mut block = Block::new();
        let res = block.set_data(&[11;1000], 17);
        assert_eq!(res, Err("Data size 1000 + offset 17 = 1017 exceeds block capacity (1016)".to_string()));


    }

}