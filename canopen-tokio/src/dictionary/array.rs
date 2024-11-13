use std::collections::HashMap;

use super::{
    dict::{Properties, SubIndex},
    Variable,
};

#[derive(Clone, Debug)]
pub struct Array {
    pub name: String,
    pub index: u16,
    pub storage_location: String,
    pub index_to_variable: HashMap<u8, Variable>,
    pub name_to_index: HashMap<String, u8>,
}

impl Array {
    pub fn new(name: &str, index: u16, properties: &Properties) -> Self {
        let storage_location = properties
            .get("StorageLocation")
            .cloned()
            .unwrap_or_default();

        Self {
            name: name.to_string(),
            index,
            storage_location,
            index_to_variable: HashMap::new(),
            name_to_index: HashMap::new(),
        }
    }
    pub fn push(&mut self, var: Variable) {
        let _ = self.name_to_index.insert(var.name.clone(), var.sub_index);
        let _ = self.index_to_variable.insert(var.sub_index, var);
    }

    pub fn index(&self, sub_index: SubIndex) -> Option<&Variable> {
        self.index_to_variable.get(&sub_index)
    }

    pub fn index_mut(&mut self, sub_index: SubIndex) -> Option<&mut Variable> {
        if self.index_to_variable.contains_key(&sub_index) {
            self.index_to_variable.get_mut(&sub_index)
        } else if sub_index > 0 && sub_index < 0xFF {
            // TODO(zephyr): copy from python impl, which doesn't follow the spec very well.
            // Please read <CANopen CiA 306> section 4.5.2.4 for details.
            let var = {
                let base_var = self.index_to_variable.get(&1).cloned()?;
                Variable {
                    name: format!("{}_{}", self.name, sub_index),
                    sub_index,
                    ..base_var
                }
            };

            self.name_to_index.insert(var.name.clone(), var.sub_index);

            let _ = self.index_to_variable.insert(var.sub_index, var);

            self.index_to_variable.get_mut(&sub_index)
        } else {
            None
        }
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Variable> {
        log::debug!("{:?}", self.name_to_index);
        let index = self.name_to_index.get(name)?;
        self.index_to_variable.get(index)
    }
}
