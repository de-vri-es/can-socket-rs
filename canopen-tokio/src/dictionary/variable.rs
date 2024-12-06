use std::str::FromStr;

use crate::{
    dictionary::{dict::format_properties_value, parse_number},
    pdo::PdoMapping,
    ObjectIndex,
};

use super::{dict::Properties, AccessType, DataType, Value};

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: String,
    pub storage_location: String,
    pub data_type: DataType,
    pub value: Value,
    pub min: Option<Value>,
    pub max: Option<Value>,
    pub pdo_mappable: bool,
    pub access_type: AccessType,
    pub parameter_value: Option<Value>,
    pub index: u16,
    pub sub_index: u8,
}

impl Variable {
    pub fn new(
        properties: &Properties,
        node_id: u8,
        name: &str,
        index: u16,
        sub_index: Option<u8>,
    ) -> Self {
        let storage_location = properties
            .get("StorageLocation")
            .cloned()
            .unwrap_or_default();

        let access_type = properties
            .get("AcessType")
            .map(|line| AccessType::from_str(line).unwrap())
            .unwrap_or(AccessType::READ_WRITE);

        let pdo_mapping = properties
            .get("PDOMapping")
            .unwrap_or(&String::from("0"))
            .parse::<i32>()
            .unwrap_or(0)
            != 0;

        let dt = properties
            .get("DataType")
            .map(|line| parse_number(line))
            .map(DataType::from_u32)
            .expect("DataType is not present in dict");

        let min = format_properties_value(properties, "LowLimit", node_id, dt);

        let max = format_properties_value(properties, "HighLimit", node_id, dt);

        let default_value = format_properties_value(properties, "DefaultValue", node_id, dt)
            .unwrap_or(Value::from_bytes(&dt.as_default_bytes()));

        let parameter_value = format_properties_value(properties, "ParameterValue", node_id, dt);

        Variable {
            name: name.to_owned(),
            storage_location,
            data_type: dt,
            access_type,
            pdo_mappable: pdo_mapping,
            min,
            max,
            value: default_value,
            parameter_value,
            index,
            sub_index: sub_index.unwrap_or(0),
        }
    }

    pub fn as_mapping(&self) -> Option<PdoMapping> {
        if !self.pdo_mappable {
            return None;
        }

        Some(PdoMapping {
            object: ObjectIndex::new(self.index, self.sub_index),
            bit_length: self.data_type.bit_size(),
        })
    }
}
