//! Fast Process URI / Resource URN parsing, NL dispatch and GBNF grammar generation.

pub mod gbnf;
pub mod nl2uri;
pub mod registry;
pub mod uri;

pub use gbnf::export_gbnf;
pub use nl2uri::{dispatch_nl_to_uri, NlIntent};
pub use registry::{ActionDefinition, ProcessUriRegistry};
pub use uri::{parse_uri_json_line, ProcessUri, ProcessUriError};
