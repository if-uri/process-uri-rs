//! Fast Process URI / Resource URN parsing, NL dispatch and GBNF grammar generation.

pub mod ffi;
pub mod gbnf;
pub mod nl2uri;
pub mod registry;
pub mod uri;

pub use ffi::{process_uri_free_string, process_uri_nl_dispatch_json, process_uri_parse_json};
pub use gbnf::export_gbnf;
pub use nl2uri::{dispatch_nl_to_uri, NlIntent};
pub use registry::{ActionDefinition, ProcessUriRegistry};
pub use uri::{parse_uri_json_line, ProcessUri, ProcessUriError};
