/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// Test metadata handling and type information generation

pub struct MetadataStruct {
    pub name: String,
    pub version: u32,
    pub features: Vec<String>,
}

pub struct MetadataObject {
    name: String,
    version: u32,
}

impl MetadataObject {
    pub fn new(name: String) -> Self {
        MetadataObject { name, version: 0 }
    }
    
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    
    pub fn set_version(&self, version: u32) {
        // In a real implementation, this would be mutable
        // For testing purposes, we'll just validate the call works
    }
    
    pub fn get_version(&self) -> u32 {
        self.version
    }
}

pub enum MetadataType {
    Basic,
    Advanced,
    Experimental,
}

pub fn test_metadata() {
    // Basic function to test metadata generation
}

pub fn get_metadata_struct() -> MetadataStruct {
    MetadataStruct {
        name: "uniffi-dart".to_string(),
        version: 1,
        features: vec!["metadata".to_string(), "testing".to_string()],
    }
}

uniffi::include_scaffolding!("api");
