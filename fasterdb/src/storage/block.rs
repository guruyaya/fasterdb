use crate::storage::serialization::ToBytes;


// Block structure for file storage
#[derive(Debug, Clone)]
pub struct Block {
    id: u64,
    data: [u8; 1008],
    next_block_offset: u64,
}

impl Default for Block {
    fn default() -> Self {
        Self {
            id: 0,
            data: [0u8; 1008],
            next_block_offset: 0, // 0 means no next block
        }
    }
}

impl Block {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn get_id(&self) -> u64 {
        self.id
    }

    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }

    pub fn set_index(&mut self) {
        self.id = u64::MAX;
    }
    
    pub fn set_deleted(&mut self) {
        self.id = 0;
    }

    pub fn is_deleted(&self) -> bool {
        self.get_id() == 0
    }
    
    pub fn is_index(&self) -> bool {
        self.get_id() == u64::MAX
    }

    pub fn set_data(&mut self, data: &[u8], offset: usize) -> Result<(), String> {
        if offset + data.len() > 1008 {
            let data_len = data.len();
            let total_size = offset + data.len();
            return Err(format!("Data size {data_len} + offset {offset} = {total_size} exceeds block capacity (1008)"));
        }
        
        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    
    pub fn get_data(&self, size: usize, offset: usize) -> Result<Vec<u8>, String> {
        if size + offset > 1008 {
            return Err("Data requested, exeeds 1008".to_string());
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

impl ToBytes for Block {
    fn to_bytes_vec(&self) -> Vec<u8> {
        match self.id {
            0 => {vec![0u8;1024]},
            _ => {
                let mut out_vec = self.get_id().to_bytes_vec();
                out_vec.extend(self.data);
                let fill_in_size = 1016 - out_vec.len();
                out_vec.extend(vec![0u8;fill_in_size]);
                out_vec.extend(self.get_next_block().to_bytes_vec());
                out_vec
            }
        }
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
        assert_eq!(block.get_id(), 0);
        assert_eq!(block.is_deleted(), true);
        assert_eq!(block.is_index(), false);
    }

    #[test]
    fn test_block_io_values_export_to_bytes() {
        let mut block = Block::new();
        block.set_id(12);
        let data = String::from("I am").to_bytes_vec();
        let out_res = block.set_data(&data, 10);
        block.set_next_block(1000);
        
        assert_eq!(out_res, Ok(()));

        let out_res = block.get_data(80, 3);
        let result = out_res.unwrap();
        assert_eq!(result.len(), 80);
        assert_eq!(block.get_id(), 12);
        assert_eq!(block.get_next_block(), 1000);
        assert!(&result[0..7].iter().fold(true, |v1, v2| -> bool {v1 && (v2 == &0)}), "Not all values are 0");
        assert_eq!(&result[7..20], vec![4, 0, 0, 0, 0, 0, 0, 0, 73, 32, 97, 109, 0]);
        assert!(&result[20..].iter().fold(true, |v1, v2| -> bool {v1 && (v2 == &0)}), "Not all values are 0");

        let bytes_vec = block.to_bytes_vec();
        assert_eq!(bytes_vec.len(), 1024);
        // check id portion
        assert_eq!(bytes_vec[0], 12);
        assert_eq!(bytes_vec[1..8], [0;7]); 

        // test offset
        assert_eq!(bytes_vec[8..18], [0;10]);
        
        // test sting length
        assert_eq!(bytes_vec[18], 4);
        assert_eq!(bytes_vec[19..26], [0;7]);

        // test string
        assert_eq!(bytes_vec[26..30], [73, 32, 97, 109]);
        
        // test rest of the block
        assert_eq!(bytes_vec[30..1016], [0;986]);
        
        // test the next item
        assert_eq!(bytes_vec[1016..1018], [232, 3]);
        assert_eq!(bytes_vec[1018..], [0; 6]);

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
        let res = block.set_data(&[11;1008], 0);
        assert_eq!(res, Ok(()));

        let mut block = Block::new();
        let res = block.set_data(&[11], 1009);
        assert_eq!(res, Err("Data size 1 + offset 1009 = 1010 exceeds block capacity (1008)".to_string()));

        let mut block = Block::new();
        let res = block.set_data(&[11;1009], 0);
        assert_eq!(res, Err("Data size 1009 + offset 0 = 1009 exceeds block capacity (1008)".to_string()));
        
        let mut block = Block::new();
        let res = block.set_data(&[11;1000], 9);
        assert_eq!(res, Err("Data size 1000 + offset 9 = 1009 exceeds block capacity (1008)".to_string()));
    }

    #[test]
    fn test_id() {
        let mut block = Block::new();

        block.set_id(20);
        assert_eq!(block.id, 20);
        assert_eq!(block.get_id(), 20);
        
        block.set_index();
        assert_eq!(block.id, u64::MAX);
        assert_eq!(block.get_id(), u64::MAX);
        
        block.set_deleted();
        assert_eq!(block.id, 0);
        assert_eq!(block.get_id(), 0);
        
    }
}