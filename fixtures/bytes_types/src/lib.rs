
 use std::collections::HashMap;
 use std::sync::{Arc, Mutex};
 use std::time::SystemTime;

#[uniffi::export]
fn take_bytes(v: Vec<u8>) -> Vec<u8> {
    v
}

#[uniffi::export]
fn take_bytes_with_validation(v: Vec<u8>) -> Vec<u8> {
    // Validate that it's valid UTF-8 if it should be
    if let Ok(s) = std::str::from_utf8(&v) {
        println!("Valid UTF-8 string: {}", s);
    }
    v
}

#[uniffi::export]
fn take_empty_bytes() -> Vec<u8> {
    Vec::new()
}

#[uniffi::export]
fn get_test_bytes() -> Vec<u8> {
    b"Hello, UniFFI bytes!".to_vec()
}

#[uniffi::export]
fn get_binary_bytes() -> Vec<u8> {
    vec![0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD, 0xFC]
}
 
//  #[derive(Debug, PartialEq)]
//  pub struct RecordWithBytes {
//      pub some_bytes: Vec<u8>,
//  }
 
//  #[derive(Debug, Clone)]
//  pub struct ComplexBytesRecord {
//      pub name: String,
//      pub required_bytes: Vec<u8>,
//      pub optional_bytes: Option<Vec<u8>>,
//      pub bytes_list: Vec<Vec<u8>>,
//      pub bytes_map: HashMap<String, Vec<u8>>,
//      pub id: u32,
//  }
 
//  impl Default for ComplexBytesRecord {
//      fn default() -> Self {
//          Self {
//              name: "default".to_string(),
//              required_bytes: Vec::new(),
//              optional_bytes: None,
//              bytes_list: Vec::new(),
//              bytes_map: HashMap::new(),
//              id: 0,
//          }
//      }
//  }
 
//  #[derive(Debug, Clone)]
//  pub struct RecordWithBytesDefaults {
//      pub name: String,
//      pub data: Vec<u8>,
//      pub optional_data: Option<Vec<u8>>,
//  }
 
//  impl Default for RecordWithBytesDefaults {
//      fn default() -> Self {
//          Self {
//              name: "default".to_string(),
//              data: b"default_data".to_vec(),
//              optional_data: Some(b"default_optional".to_vec()),
//          }
//      }
//  }
 
//  #[uniffi::export]
// fn make_record_with_bytes() -> RecordWithBytes {
//     RecordWithBytes {
//         some_bytes: vec![0, 1, 2, 3, 4],
//     }
// }

// #[uniffi::export]
// fn take_record_with_bytes(rwb: RecordWithBytes) -> Vec<u8> {
//     rwb.some_bytes
// }

// #[uniffi::export]
// fn create_complex_bytes_record() -> ComplexBytesRecord {
//      ComplexBytesRecord {
//          name: "test_record".to_string(),
//          required_bytes: b"required_data".to_vec(),
//          optional_bytes: Some(b"optional_data".to_vec()),
//          bytes_list: vec![
//              b"first".to_vec(),
//              b"second".to_vec(),
//              vec![0x01, 0x02, 0x03],
//          ],
//          bytes_map: [
//              ("key1".to_string(), b"value1".to_vec()),
//              ("key2".to_string(), b"value2".to_vec()),
//              ("binary".to_string(), vec![0xFF, 0xFE, 0xFD]),
//          ]
//          .into_iter()
//          .collect(),
//          id: 42,
//      }
//  }
 
//  #[uniffi::export]
// fn create_empty_bytes_record() -> ComplexBytesRecord {
//     ComplexBytesRecord {
//         name: "empty_record".to_string(),
//         required_bytes: Vec::new(),
//         optional_bytes: None,
//         bytes_list: Vec::new(),
//         bytes_map: HashMap::new(),
//         id: 0,
//     }
// }
 
//  pub struct BytesProcessor {
//      name: String,
//      buffer: Mutex<Vec<u8>>,
//  }
 
//  impl BytesProcessor {
//      pub fn new(name: String) -> Self {
//          Self {
//              name,
//              buffer: Mutex::new(Vec::new()),
//          }
//      }
 
//      pub fn get_name(&self) -> String {
//          self.name.clone()
//      }
 
//      /// Reverses the bytes - equivalent to coverall fixture
//      pub fn reverse(&self, mut value: Vec<u8>) -> Vec<u8> {
//          value.reverse();
//          value
//      }
 
//      /// Concatenates multiple byte arrays
//      pub fn concat(&self, arrays: Vec<Vec<u8>>) -> Vec<u8> {
//          arrays.into_iter().flatten().collect()
//      }
 
//      /// Splits bytes at delimiter
//      pub fn split_bytes(&self, data: Vec<u8>, delimiter: u8) -> Vec<Vec<u8>> {
//          let mut result = Vec::new();
//          let mut current = Vec::new();
         
//          for byte in data {
//              if byte == delimiter {
//                  result.push(current);
//                  current = Vec::new();
//              } else {
//                  current.push(byte);
//              }
//          }
         
//          if !current.is_empty() {
//              result.push(current);
//          }
         
//          result
//      }
 
//      /// Stores bytes in internal buffer
//      pub fn store_bytes(&self, data: Vec<u8>) {
//          *self.buffer.lock().unwrap() = data;
//      }
 
//      /// Retrieves bytes from internal buffer
//      pub fn retrieve_bytes(&self) -> Vec<u8> {
//          self.buffer.lock().unwrap().clone()
//      }
 
//      /// Processes bytes and returns a record
//      pub fn process_to_record(&self, data: Vec<u8>) -> RecordWithBytes {
//          let mut processed = data;
//          processed.reverse();
//          RecordWithBytes {
//              some_bytes: processed,
//          }
//      }
 
//      /// XOR operation on bytes
//      pub fn xor_bytes(&self, data: Vec<u8>, key: u8) -> Vec<u8> {
//          data.into_iter().map(|b| b ^ key).collect()
//      }
 
//      /// Get bytes statistics
//      pub fn get_stats(&self, data: Vec<u8>) -> HashMap<String, u64> {
//          let mut stats = HashMap::new();
//          stats.insert("length".to_string(), data.len() as u64);
//          stats.insert("sum".to_string(), data.iter().map(|&b| b as u64).sum());
//          stats.insert("max".to_string(), data.iter().map(|&b| b as u64).max().unwrap_or(0));
//          stats.insert("min".to_string(), data.iter().map(|&b| b as u64).min().unwrap_or(0));
//          stats
//      }
//  }
 
//  // ============================================================================
//  // Callback Interface with Bytes
//  // ============================================================================
 
//  pub trait BytesCallbackInterface {
//      fn do_nothing(&self);
//      fn process_bytes(&self, data: Vec<u8>) -> Vec<u8>;
//      fn with_bytes_record(&self, record: RecordWithBytes) -> Vec<u8>;
//      fn combine_bytes(&self, first: Vec<u8>, second: Vec<u8>) -> Vec<u8>;
//      fn validate_bytes(&self, data: Vec<u8>) -> bool;
//      fn get_bytes_info(&self) -> HashMap<String, Vec<u8>>;
//  }
 
//  pub trait BytesAsyncCallbackInterface {
//      fn process_bytes_async(&self, data: Vec<u8>) -> Vec<u8>;
//      fn batch_process(&self, batches: Vec<Vec<u8>>) -> Vec<Vec<u8>>;
//  }
 
//  // ============================================================================
//  // Functions Testing Callback Interfaces
//  // ============================================================================
 
//  fn call_bytes_callback_interface(cb: Box<dyn BytesCallbackInterface>) {
//      cb.do_nothing();
     
//      // Test basic bytes processing
//      let test_data = vec![1, 2, 3, 4, 5];
//      let result = cb.process_bytes(test_data.clone());
//      assert_eq!(result.len(), test_data.len());
     
//      // Test with record
//      let record = RecordWithBytes {
//          some_bytes: vec![9, 8, 7],
//      };
//      let record_result = cb.with_bytes_record(record);
//      assert_eq!(record_result, vec![9, 8, 7]);
     
//      // Test combining bytes
//      let combined = cb.combine_bytes(vec![1, 2], vec![3, 4]);
//      assert_eq!(combined, vec![1, 2, 3, 4]);
     
//      // Test validation
//      assert!(cb.validate_bytes(vec![1, 2, 3]));
     
//      // Test info retrieval
//      let info = cb.get_bytes_info();
//      assert!(!info.is_empty());
//  }
 
//  // ============================================================================
//  // Error Handling with Bytes
//  // ============================================================================
 
//  #[derive(Debug, thiserror::Error)]
//  pub enum BytesError {
//      #[error("Invalid bytes length: {length}")]
//      InvalidLength { length: usize },
//      #[error("Invalid byte value: {value}")]
//      InvalidValue { value: u8 },
//      #[error("Processing failed")]
//      ProcessingFailed,
//  }
 
//  fn validate_bytes_length(data: Vec<u8>, min_length: usize) -> Result<Vec<u8>, BytesError> {
//      if data.len() < min_length {
//          Err(BytesError::InvalidLength { length: data.len() })
//      } else {
//          Ok(data)
//      }
//  }
 
//  fn validate_bytes_content(data: Vec<u8>) -> Result<Vec<u8>, BytesError> {
//      if data.iter().any(|&b| b > 127) {
//          Err(BytesError::InvalidValue { value: *data.iter().find(|&&b| b > 127).unwrap() })
//      } else {
//          Ok(data)
//      }
//  }
 
//  fn fallible_bytes_processing(data: Vec<u8>) -> Result<Vec<u8>, BytesError> {
//      if data.is_empty() {
//          Err(BytesError::ProcessingFailed)
//      } else {
//          Ok(data.into_iter().map(|b| b.wrapping_add(1)).collect())
//      }
//  }
 
//  // ============================================================================
//  // Enum with Bytes
//  // ============================================================================
 
//  #[derive(Debug, Clone)]
//  pub enum BytesEnum {
//      Empty,
//      Small(Vec<u8>),
//      Large { data: Vec<u8>, metadata: String },
//      Multiple(Vec<Vec<u8>>),
//  }
 
//  fn create_bytes_enum_variants() -> Vec<BytesEnum> {
//      vec![
//          BytesEnum::Empty,
//          BytesEnum::Small(vec![1, 2, 3]),
//          BytesEnum::Large {
//              data: b"large_data".to_vec(),
//              metadata: "metadata".to_string(),
//          },
//          BytesEnum::Multiple(vec![
//              vec![1, 2],
//              vec![3, 4],
//              vec![5, 6],
//          ]),
//      ]
//  }
 
//  fn process_bytes_enum(variant: BytesEnum) -> Vec<u8> {
//      match variant {
//          BytesEnum::Empty => Vec::new(),
//          BytesEnum::Small(data) => data,
//          BytesEnum::Large { data, .. } => data,
//          BytesEnum::Multiple(arrays) => arrays.into_iter().flatten().collect(),
//      }
//  }
 
//  // ============================================================================
//  // Utility Functions
//  // ============================================================================
 
//  /// Test bytes with various encodings
//  fn test_bytes_encodings() -> HashMap<String, Vec<u8>> {
//      let mut encodings = HashMap::new();
     
//      encodings.insert("utf8".to_string(), "Hello, 世界!".as_bytes().to_vec());
//      encodings.insert("ascii".to_string(), b"Hello, World!".to_vec());
//      encodings.insert("binary".to_string(), vec![0x00, 0x01, 0x02, 0xFF, 0xFE, 0xFD]);
//      encodings.insert("empty".to_string(), Vec::new());
//      encodings.insert("single".to_string(), vec![42]);
//      encodings.insert("large".to_string(), vec![0x55; 1024]);
     
//      encodings
//  }
 
//  /// Test bytes with extreme sizes
//  fn test_bytes_sizes() -> HashMap<String, Vec<u8>> {
//      let mut sizes = HashMap::new();
     
//      sizes.insert("empty".to_string(), Vec::new());
//      sizes.insert("tiny".to_string(), vec![1]);
//      sizes.insert("small".to_string(), vec![1; 10]);
//      sizes.insert("medium".to_string(), vec![2; 1000]);
//      sizes.insert("large".to_string(), vec![3; 100000]);
     
//      sizes
//  }
 
//  /// Comprehensive bytes test that combines multiple patterns
//  fn comprehensive_bytes_test() -> ComplexBytesRecord {
//      let record = create_complex_bytes_record();
     
//      // Test processing
//      let processor = BytesProcessor::new("test_processor".to_string());
//      let reversed = processor.reverse(record.required_bytes.clone());
//      let concatenated = processor.concat(record.bytes_list.clone());
     
//      // Create result record
//      ComplexBytesRecord {
//          name: "comprehensive_test".to_string(),
//          required_bytes: reversed,
//          optional_bytes: Some(concatenated),
//          bytes_list: vec![
//              b"test1".to_vec(),
//              b"test2".to_vec(),
//          ],
//          bytes_map: [
//              ("reversed".to_string(), reversed),
//              ("concatenated".to_string(), concatenated),
//          ]
//          .into_iter()
//          .collect(),
//          id: 999,
//      }
//  }

 uniffi::include_scaffolding!("api");