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
            name_to_index: HashMap::new()
        }
    }
    pub fn push(&mut self, var: Variable) {
        let _ = self.name_to_index.insert(var.name.clone(), var.sub_index);
        let _ = self.index_to_variable.insert(var.sub_index, var);
    }

    pub fn get(&self, sub_index: SubIndex) -> Option<&Variable> {
        todo!()
    }

    pub fn get_mut<'a>(
        &'a mut self,
        sub_index: SubIndex,
    ) -> Option<&'a mut Variable> {
        if let Some(ref value) = self.index_to_variable.get_mut(&sub_index) {
            return Some(value);
        }

        //compiler will optimize
        if sub_index > 0 && sub_index < 0xFF {
            // TODO(zephyr): copy from python impl, which doesn't follow the spec very well.
            // Please read <CANopen CiA 306> section 4.5.2.4 for details.
            let base_var = self.index_to_variable.get(&1)?;
            let new_var = Variable {
                name: format!("{}_{}", self.name, sub_index),
                sub_index,
                ..base_var.clone()
            };

            self.push(new_var);
            self.index_to_variable.get_mut(&sub_index)
        } else {
            None
        }
    }
}
