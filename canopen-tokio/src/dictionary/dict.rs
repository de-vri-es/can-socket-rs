use std::collections::HashMap;
use std::str::FromStr;
use std::string::ToString;

use crate::dictionary::AccessType;
use ini_core as ini;

use super::{parse_number, Array, ObjectType, Record};
use super::{DataType, Variable};
use super::{LoadError, Value};

/// Object Types
const OBJECT_TYPE_VARIABLE: u32 = 7;
const OBJECT_TYPE_ARRAY: u32 = 8;
const OBJECT_TYPE_RECORD: u32 = 9;

pub type SubIndex = u8;
pub type Properties = HashMap<String, String>;

#[derive(Clone, Debug)]
pub struct ObjectDirectory {
    pub node_id: u8,
    pub index_to_object: HashMap<u16, ObjectType>,
    pub name_to_index: HashMap<String, u16>,
}

impl ObjectDirectory {
    pub fn load_from_content(
        node_id: u8,
        content: &str,
    ) -> Result<Self, LoadError> {
        let mut dict = ObjectDirectory {
            node_id,
            index_to_object: HashMap::new(),
            name_to_index: HashMap::new(),
        };

        let mut current_section_name: Option<String> = None;
        let mut current_properties: HashMap<String, String> = HashMap::new();

        for item in ini::Parser::new(content) {
            match item {
                ini::Item::Section(name) => {
                    if let Some(section_name) = current_section_name.take() {
                        // Get all properties, process the section.
                        dict.process_section(
                            &section_name,
                            &current_properties,
                        )?;
                        current_properties.clear();
                    }
                    current_section_name = Some(String::from(name));
                }
                ini::Item::Property(key, maybe_value) => {
                    let value = String::from(maybe_value.unwrap_or_default());
                    current_properties.insert(String::from(key), value);
                }
                _ => {} // Ignore for other sections, for example comments / section end.
            }
        }

        // The last section
        if let Some(section_name) = current_section_name {
            dict.process_section(&section_name, &current_properties)?
        }

        Ok(dict)
    }

    pub fn node_id(&self) -> u8 {
        self.node_id
    }
}

impl ObjectDirectory {
    pub fn push(&mut self, index: u16, name: String, obj: ObjectType) {
        self.index_to_object.insert(index, obj);
        self.name_to_index.insert(name, index);
    }

    pub fn push_by_sub_index(
        &mut self,
        index: u16,
        var: Variable,
    ) -> Result<(), String> {
        match self.index_to_object.get_mut(&index) {
            None => Err(format!("No id:{:x?}", index)),

            Some(ObjectType::Record(record)) => {
                record.push(var);
                Ok(())
            }

            Some(ObjectType::Array(array)) => {
                array.push(var);
                Ok(())
            }
            _ => Err("no subindex for a Variable object".to_string()),
        }
    }

    pub fn set(
        &mut self,
        index: u16,
        sub_index: Option<SubIndex>,
        data: &[u8],
    ) -> Result<(), String> {
        let Some(var) = self.get_mut(index, sub_index) else {
            log::warn!(
                "No value by index={index} and subindex={:?}",
                sub_index
            );
            return Err("sdfa".to_string());
        };

        if !var.access_type.is_writable() {
            log::warn!("Failed to set not writable");
            return Err("sdfa".to_string());
        }

        if var.data_type.size() != data.len() {
            log::warn!("Failed to set data. Invalid length");
            return Err("sdfa".to_string());
        }

        var.value.set_data(data[0..var.data_type.size()].to_vec());

        Ok(())
    }

    pub fn get(
        &mut self,
        index: u16,
        sub_index: Option<SubIndex>,
    ) -> Option<&Variable> {
        let kind = self.index_to_object.get(&index)?;
        match kind {
            ObjectType::Variable(var) => Some(var),

            ObjectType::Array(array) => {
                let index = sub_index.expect("SubIndex is not provided");
                array.get(index)
            }

            ObjectType::Record(record) => {
                let index = sub_index.expect("SubIndex is not provided");
                record.get(index)
            }
        }
    }

    pub fn get_mut(
        &mut self,
        index: u16,
        sub_index: Option<SubIndex>,
    ) -> Option<&mut Variable> {
        let kind = self.index_to_object.get_mut(&index)?;

        match kind {
            ObjectType::Variable(var) => Some(var),
            ObjectType::Array(array) => {
                let index = sub_index.expect("SubIndex is not provided");
                array.get_mut(index)
            }
            ObjectType::Record(record) => {
                let index = sub_index.expect("SubIndex is not provided");
                record.get_mut(index)
            }
        }
    }

    pub fn get_object_by_name(&self, name: &str) -> Option<&ObjectType> {
        let index = self.name_to_index.get(name)?;
        self.index_to_object.get(index)
    }

    pub fn get_mut_object(&mut self, index: u16) -> Option<&mut ObjectType> {
        self.index_to_object.get_mut(&index)
    }

    fn process_section(
        &mut self,
        section_name: &str,
        properties: &HashMap<String, String>,
    ) -> Result<(), LoadError> {
        fn is_hex_char(c: char) -> bool {
            c.is_ascii_digit()
                || ('a'..='f').contains(&c)
                || ('A'..='F').contains(&c)
        }

        fn is_top(s: &str) -> bool {
            s.len() == 4 && s.chars().all(is_hex_char)
        }

        fn is_sub(s: &str) -> Option<(u16, u8)> {
            if s.len() > 7
                && s[4..7].eq_ignore_ascii_case("sub")
                && s[0..4].chars().all(is_hex_char)
            {
                let (index_str, sub_str) = (&s[0..4], &s[7..]);
                match (
                    u16::from_str_radix(index_str, 16),
                    u8::from_str(sub_str),
                ) {
                    (Ok(index), Ok(sub)) => Some((index, sub)),
                    _ => None,
                }
            } else {
                None
            }
        }

        fn is_name(s: &str) -> Option<u16> {
            s.ends_with("Name")
                .then(|| s[0..4].chars().all(is_hex_char))
                .and_then(|valid| {
                    valid.then(|| u16::from_str_radix(&s[0..4], 16).ok())
                })
                .flatten()
        }

        if is_top(section_name) {
            let Ok(index) = u16::from_str_radix(section_name, 16) else {
                return Err(format!("Invalid index: {section_name}").into());
            };

            let Some(name) = properties.get("ParameterName") else {
                return Err(format!("{section_name}. No Parameter Name").into());
            };

            let Some(object_kind) =
                properties.get("ObjectType").map(|v| parse_number::<u32>(v))
            else {
                return Err(format!("{section_name}. No Object type").into());
            };

            match object_kind {
                OBJECT_TYPE_VARIABLE => {
                    let var = Variable::new(
                        properties,
                        self.node_id,
                        name,
                        index,
                        None,
                    );

                    self.name_to_index.insert(var.name.clone(), index);
                    self.index_to_object
                        .insert(index, ObjectType::Variable(var));
                }

                OBJECT_TYPE_ARRAY => {
                    let mut array = Array::new(name, index, properties);

                    if properties.contains_key("CompactSubObj") {
                        let last_subindex = Variable {
                            name: "Number of entries".to_string(),
                            index,
                            sub_index: 0,
                            data_type: DataType::Unsigned8,
                            value: Value::from_bytes(&0u32.to_le_bytes()),
                            min: None,
                            max: None,
                            pdo_mappable: false,
                            access_type: AccessType::NONE,
                            storage_location: "".to_string(),
                            parameter_value: None,
                        };

                        array.push(last_subindex);
                        array.push(Variable::new(
                            properties,
                            self.node_id,
                            name,
                            index,
                            Some(1u8),
                        ));
                    }

                    self.push(index, name.clone(), ObjectType::Array(array));
                }
                OBJECT_TYPE_RECORD => {
                    let record = Record::new(name, index, properties);
                    self.name_to_index.insert(name.clone(), index);
                    self.index_to_object
                        .insert(index, ObjectType::Record(record));
                }
                _ => { // ignore
                }
            }
        } else if let Some((index, sub_index)) = is_sub(section_name) {
            let Some(name) = properties.get("ParameterName") else {
                return Err(format!("{section_name}. No name").into());
            };

            let variable = Variable::new(
                properties,
                self.node_id,
                name,
                index,
                Some(sub_index),
            );

            self.push_by_sub_index(index, variable).map_err(|cause| {
                LoadError::from(format!(
                    "{section_name}. Failed to add variable: {cause}"
                ))
            })?;
        } else if let Some(index) = is_name(section_name) {
            // Logic related to CompactSubObj
            let Some(num_of_entries) = properties
                .get("NrOfEntries")
                .and_then(|v| v.parse::<u8>().ok())
            else {
                return Err(
                    format!("{section_name}. NrOfEntries is invalid").into()
                );
            };

            if let Some(ObjectType::Array(arr)) =
                self.index_to_object.get_mut(&index)
            {
                if let Some(src_var) = arr.index_to_variable.get(&1u8) {
                    let cloned_src_var = src_var.clone();
                    let mut new_vars = Vec::new();
                    for subindex in 1..=num_of_entries {
                        let mut var = cloned_src_var.clone();
                        if let Some(name) =
                            properties.get(&subindex.to_string())
                        {
                            var.name = name.clone();
                            var.sub_index = subindex;
                            new_vars.push(var);
                        }
                    }
                    for var in new_vars {
                        arr.push(var);
                    }
                }
            }
        }

        Ok(())
    }
}

pub fn evaluate_expression_with_node_id(
    node_id: u8,
    expression: &str,
) -> String {
    // Replace $NODEID with the actual node_id
    let modified_expression =
        expression.replace("$NODEID", &node_id.to_string());

    // Evaluate simple arithmetic expressions
    modified_expression
        .split('+')
        .map(|s| s.trim())
        .filter_map(|s| {
            if s.starts_with("0x") || s.starts_with("0X") {
                i64::from_str_radix(&s[2..], 16).ok()
            } else {
                s.parse::<i64>().ok()
            }
        })
        .sum::<i64>()
        .to_string()
}

pub fn format_properties_value(
    properties: &HashMap<String, String>,
    property_name: &str,
    node_id: u8,
    kind: DataType,
) -> Option<Value> {
    let raw = match properties.get(property_name) {
        Some(value) if !value.is_empty() => value,
        _ => return None,
    };

    let modified_raw = if raw.contains("$NODEID") {
        evaluate_expression_with_node_id(node_id, raw)
    } else {
        raw.clone()
    };

    match Value::from_str(&modified_raw, kind) {
        Ok(val) => Some(val),
        Err(e) => {
            log::error!("Error converting string to value: {:?}", e);
            None
        }
    }
}
