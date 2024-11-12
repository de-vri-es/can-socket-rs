mod access;
mod data_type;
mod dict;
mod number;
mod record;
mod value;
mod variable;
mod array;
mod error;
mod object;

pub use access::*;
pub use data_type::*;
pub use number::*;
pub use record::*;
pub use value::*;
pub use variable::*;
pub use array::*;
pub use error::*;
pub use object::*;
pub use dict::*;

// TODO(zephyr): Split the logic to read EDS and object_directory. Tasks:
//   - Make Array fixed, and provide real get_variable() / get_mut_variable().
//   - Serialize the OD as binary, provide tools to read it
//   - Read the serialized binary as input instead of EDS / DCF
//   - The conversion of EDS <=> binary should happen outside of the main server.
