use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use crate::nl2uri::dispatch_nl_to_uri;
use crate::uri::ProcessUri;

/// Parse a Process URI string and return a JSON serialized string.
/// The caller is responsible for freeing the returned string using `process_uri_free_string`.
#[no_mangle]
pub extern "C" fn process_uri_parse_json(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(input) };
    let input_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    match ProcessUri::parse(input_str) {
        Ok(uri) => {
            let json = serde_json::to_string(&uri).unwrap_or_default();
            CString::new(json).unwrap_or_default().into_raw()
        }
        Err(e) => {
            let err_json = format!("{{\"error\":\"{}\"}}", e);
            CString::new(err_json).unwrap_or_default().into_raw()
        }
    }
}

/// Dispatch natural language text to a matching Process URI intent and return a JSON serialized string.
/// The caller is responsible for freeing the returned string using `process_uri_free_string`.
#[no_mangle]
pub extern "C" fn process_uri_nl_dispatch_json(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }
    let c_str = unsafe { CStr::from_ptr(input) };
    let input_str = match c_str.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };
    let (intent, uri) = dispatch_nl_to_uri(input_str);
    let payload = serde_json::json!({
        "intent": format!("{:?}", intent),
        "uri": uri.canonical_action(),
        "scheme": uri.scheme,
        "domain": uri.domain,
        "action": uri.action_or_resource,
        "query_params": uri.query_params,
        "is_urn": uri.is_urn,
    });
    let json = serde_json::to_string(&payload).unwrap_or_default();
    CString::new(json).unwrap_or_default().into_raw()
}

/// Free a CString allocated by `process_uri_parse_json` or `process_uri_nl_dispatch_json`.
#[no_mangle]
pub extern "C" fn process_uri_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
