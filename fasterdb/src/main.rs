use std::collections::HashMap;

#[derive(Debug)]
enum OperationError {
    KeyMissing
    
}
#[derive(Debug)]
struct DataBase {
    documents: HashMap<u64, String>,
    next_id: u64
}

impl DataBase {
    fn from_hashmap(documents: HashMap<u64, String>) -> Self {
        let binding     = documents.clone();
        let next_val = binding.keys().into_iter().max().unwrap_or(&1);
        Self {
            documents: documents,
            next_id: *next_val
        }
    }
    fn new() -> Self{
        Self::from_hashmap(HashMap::new())
    }

    fn write(&mut self, value: String) -> Result<u64, OperationError> {
        let ret_val = self.next_id.clone();
        self.documents.insert(self.next_id, value);
        self.next_id += 1;

        Ok(ret_val)
    }

    fn read(&self, key: u64) -> Result<Option<&String>, OperationError> {
        Ok(self.documents.get(&key))
    }

    fn update(&mut self, key: u64, new_value: &String) -> Result<u64, OperationError>{
        let value_wrapped = self.documents.get_mut(&key); 
        match value_wrapped {
            Some(value) => {*value = String::from(new_value); Ok(key)},
            None => Err(OperationError::KeyMissing)
        }
    }

}
fn main() {
    let mut database = DataBase::new();
    println!("{:?}", database);
    if let Ok(id) =  database.write(String::from("{\"hello\": \"world\"}")) {
        println!("{:?}", id);
    }

    for testkey in 1..=2 {
        let out = database.read(testkey);
        match out {
            Ok(Some(val)) => {println!("{}", &val);},
            Ok(None) => {println!("Found noting!!");},
            Err(_) => {println!("Something terrible just happen")}
        }
    }

    for testkey in 1..=2 {
        let out = database.update(testkey, &String::from("{\"Bye\":\"City\"}"));
        match out {
            Ok(val) => {
                println!("Updated {}", &val);
                if let Ok(Some(new_value)) = database.read(testkey) {

                    println!("{}", new_value)
                }else{
                    println!("Why are we here? This should never happen!!!")
                }
            },
            Err(e) => {println!("Got error {:?}", e)}
        }
    }
}
