use std::collections::HashMap;

use super::{dict::Properties, Variable};

#[derive(Clone, Debug)]
pub struct Record {
    pub name: String,
    pub index: u16,
    pub storage_location: String,
    pub index_to_variable: HashMap<u8, Variable>,
    pub name_to_index: HashMap<String, u8>,
}

impl Record {
    pub fn new(name: &str, index: u16, properties: &Properties) -> Self {
        let storage_location = properties
            .get("StorageLocation")
            .cloned()
            .unwrap_or_default();

        Self {
            name: name.to_string(),
            index,
            storage_location,
            name_to_index: HashMap::new(),
            index_to_variable: HashMap::new(),
        }
    }

    pub fn push(&mut self, var: Variable) {
        let _ = self.name_to_index.insert(var.name.clone(), var.sub_index);
        let _ = self.index_to_variable.insert(var.sub_index, var);
    }

    pub fn get(&self, sub_index: u8) -> Option<&Variable> {
        self.index_to_variable.get(&sub_index)
    }

    pub fn get_mut(&mut self, sub_index: u8) -> Option<&mut Variable> {
        self.index_to_variable.get_mut(&sub_index)
    }

    pub fn get_by_name(&self, name: &str) -> Option<&Variable> {
        let index = *self.name_to_index.get(name)?;
        self.index_to_variable.get(&index)
    }
}
