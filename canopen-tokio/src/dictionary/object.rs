use super::{Array, Record, Variable};

#[derive(Clone, Debug)]
pub enum ObjectType {
    Variable(Variable),
    Array(Array),
    Record(Record),
}

impl ObjectType {
    pub fn var(&self) -> Option<&Variable> {
        if let ObjectType::Variable(ref var) = self {
            return Some(var);
        }
        None
    }

    pub fn array(&self) -> Option<&Array> {
        if let ObjectType::Array(ref arr) = self {
            return Some(arr);
        }

        None
    }

    pub fn record(&self) -> Option<&Record> {
        if let ObjectType::Record(ref rec) = self {
            return Some(rec);
        }

        None
    }
}
