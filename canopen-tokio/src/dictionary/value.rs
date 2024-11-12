use super::{parse_number, DataType};

#[derive(Clone, Debug)]
pub struct Value {
    data: Vec<u8>,
}

pub fn make_error(kidn: DataType, data_string: &str) -> () {
    ()
}

impl Value {
    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    pub fn from_str(data_string: &str, kind: DataType) -> Result<Value, ()> {
        match kind {
            DataType::Unknown => Err(make_error(kind, data_string)),

            DataType::Boolean => {
                let val: u8 = match data_string.to_lowercase().as_str() {
                    "true" | "1" => 1,
                    "false" | "0" => 0,
                    _ => return Err(make_error(kind, data_string)),
                };
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Integer8 => {
                let val: i8 = parse_number(data_string);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Integer16 => {
                let val: i16 = parse_number(data_string);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Integer32 => {
                let val: i32 = parse_number(data_string);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Integer64 => {
                let val: i64 = parse_number(data_string);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Unsigned8 => {
                let val: u8 = parse_number(&data_string);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Unsigned16 => {
                let val: u16 = parse_number(data_string);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Unsigned32 => {
                let val: u32 = parse_number(data_string);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Unsigned64 => {
                let val: u64 = parse_number(data_string);
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Real32 => {
                let val: f32 = data_string
                    .parse()
                    .map_err(|_| make_error(*kind, data_string))?;
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::Real64 => {
                let val: f64 = data_string
                    .parse()
                    .map_err(|_| make_error(*kind, data_string))?;
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }

            DataType::VisibleString
            | DataType::OctetString
            | DataType::UnicodeString => Ok(Value {
                data: data_string.as_bytes().to_vec(),
            }),

            DataType::Domain => {
                let val: i32 = data_string
                    .parse()
                    .map_err(|_| make_error(*kind, data_string))?;
                Ok(Value::from_bytes(&val.to_le_bytes()))
            }
        }
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
