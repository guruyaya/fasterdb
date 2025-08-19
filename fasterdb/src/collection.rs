use std::collections::HashMap;
use crate::errors::OperationError;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Collection {
    documents: HashMap<u64, String>,
    next_id: u64
}

#[allow(dead_code)]
impl Collection {
    pub fn new() -> Self{
        Self {
            documents: HashMap::new(),
            next_id: 1
        }
    }

    pub fn write(&mut self, value: String) -> Result<u64, OperationError> {
        let ret_val = self.next_id.clone();
        self.documents.insert(self.next_id, value);
        self.next_id += 1;

        Ok(ret_val)
    }

    pub fn read(&self, key: u64) -> Result<Option<&String>, OperationError> {
        Ok(self.documents.get(&key))
    }

    pub fn update(&mut self, key: u64, new_value: &String) -> Result<u64, OperationError>{
        let value_wrapped = self.documents.get_mut(&key); 
        match value_wrapped {
            Some(value) => {*value = String::from(new_value); Ok(key)},
            None => Err(OperationError::KeyMissing)
        }
    }

    pub fn delete(&mut self, key: u64) -> Result<String, OperationError> {
        let value_wrapped = self.documents.remove(&key);
        match value_wrapped {
            Some(value) => {Ok(value)},
            None => Err(OperationError::KeyMissing)
        }
    }

    // Helper functions for testing
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn get_next_id(&self) -> u64 {
        self.next_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(unused_must_use)]
    fn setup_db() -> Collection {
        let mut collection = Collection::new();
        collection.write(String::from("Hello123"));
        collection
    }
    
    #[test]
    fn test_create_empty() {
        let collection = Collection::new();
        assert_eq!(collection.documents.len(), 0);
        assert_eq!(collection.next_id, 1);
    }
    
    #[test]
    fn test_create_entry() {
        let mut collection = Collection::new();
        let num = collection.write(String::from("Hello123"));
        assert_eq!(num.unwrap(), 1);
        assert_eq!(collection.documents.len(), 1);
        assert_eq!(collection.documents.get(&1), Some(&String::from("Hello123")));
        assert_eq!(collection.next_id, 2);
    }

    #[test]
    fn test_update_entry() {
        let mut collection = setup_db();
        let old_len = collection.documents.len();
        let update_to = String::from("Hello95");
        let num_wrapper = collection.update(1, &update_to);

        assert!(num_wrapper.is_ok());
        if let Ok(num) = num_wrapper {
            assert_eq!(collection.documents.len(), old_len);
            assert_eq!(collection.documents.get(&num), Some(&update_to));
        }
    }

    #[test]
    fn test_delete_entry() {
        let mut collection = setup_db();
        let old_len = collection.documents.len();

        let pre_del_result = collection.read(1);

        assert!(pre_del_result.is_ok());
        let pre_del_option = pre_del_result.unwrap();

        assert!(pre_del_option.is_some());
        let pre_del = pre_del_option.unwrap().clone();

        let deleted_string_result = collection.delete(1);

        assert!(deleted_string_result.is_ok());
        let deleted_string = deleted_string_result.unwrap();

        assert_eq!(collection.documents.len(), old_len - 1);
        assert_eq!(pre_del, deleted_string);
    }

    #[test]
    fn test_read_unexisting() {
        let collection = setup_db();

        let res = collection.read(12);
        assert!(res.is_ok()); // Note: missing entry is not an error

        let opt = res.unwrap();
        assert!(opt.is_none());
    }

    #[test]
    fn test_delete_unexisting() {
        let mut collection = setup_db();
        
        let result = collection.delete(99);
        assert!(matches!(result, Err(OperationError::KeyMissing)))
    }
    
    #[test]
    fn test_update_unexisting() {
        let mut collection = setup_db();
        
        let result = collection.update(99, &String::from("Yo yo yo"));
        assert!(matches!(result, Err(OperationError::KeyMissing)))
    }
}