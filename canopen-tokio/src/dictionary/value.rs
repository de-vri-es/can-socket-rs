use super::{parse_number, DataType};

#[derive(Clone, Debug)]
pub struct Value {
    data: Vec<u8>,
}

impl Value {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    pub fn from_str(raw_value: &str, kind: DataType) -> Result<Value, String> {
        match kind {
            DataType::Unknown => Err("Unknown data type!".into()),

            DataType::Boolean => {
                let val = match raw_value.to_lowercase().as_str() {
                    "true" | "1" => 1u8,
                    "false" | "0" => 0u8,
                    _ => return Err("Invalid bool value".into()),
                };

                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Integer8 => {
                let val: i8 = parse_number(raw_value);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Integer16 => {
                let val: i16 = parse_number(raw_value);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Integer32 => {
                let val: i32 = parse_number(raw_value);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Integer64 => {
                let val: i64 = parse_number(raw_value);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Unsigned8 => {
                let val: u8 = parse_number(raw_value);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Unsigned16 => {
                let val: u16 = parse_number(raw_value);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Unsigned32 => {
                let val: u32 = parse_number(raw_value);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Unsigned64 => {
                let val: u64 = parse_number(raw_value);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Real32 => {
                let Ok(val) = raw_value.parse::<f32>() else {
                    return Err("Failed to parse f32".into());
                };

                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Real64 => {
                let Ok(val) = raw_value.parse::<f64>() else {
                    return Err("Failed to parse f64".into());
                };

                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::VisibleString
            | DataType::OctetString
            | DataType::UnicodeString => Ok(Value {
                data: raw_value.as_bytes().to_vec(),
            }),

            DataType::Domain => {
                let Ok(val) = raw_value.parse::<i32>() else {
                    return Err("Failed to parse domain id".into());
                };

                Ok(Value::from_bytes(&val.to_le_bytes()))
            }
        }
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    #[allow(unused)]
    fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
