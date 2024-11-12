use super::{Array, Record, Variable};

/// Object Type
#[derive(Clone, Debug)]
pub enum ObjectType {
    /// Array
    Array(Array),
    /// Variabl
    Variable(Variable),
    /// Record
    Record(Record),
}

impl ObjectType {
    /// Returns the var of this [`ObjectType`].
    pub fn var(&self) -> Option<&Variable> {
        if let ObjectType::Variable(ref var) = self {
            return Some(var);
        }
        None
    }

    /// Returns the array of this [`ObjectType`].
    pub fn array(&self) -> Option<&Array> {
        if let ObjectType::Array(ref arr) = self {
            return Some(arr);
        }

        None
    }

    /// Returns the record of this [`ObjectType`].
    pub fn record(&self) -> Option<&Record> {
        if let ObjectType::Record(ref rec) = self {
            return Some(rec);
        }

        None
    }
}
