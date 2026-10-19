//! Fast Process URI / Resource URN parsing, NL dispatch, System-1 calibrated router, and GBNF grammar generation.

pub mod ffi;
pub mod gbnf;
pub mod nl2uri;
pub mod registry;
pub mod router_engine;
pub mod uri;

pub use ffi::{process_uri_free_string, process_uri_nl_dispatch_json, process_uri_parse_json};
pub use gbnf::export_gbnf;
pub use nl2uri::{dispatch_nl_to_uri, NlIntent};
pub use registry::{ActionDefinition, ProcessUriRegistry};
pub use router_engine::{
    evaluate_decision_choice, score_bindings_calibrated, score_candidate_calibrated,
    tokenize_calibrated, tokenize_decision,
};
pub use uri::{parse_jsonl_records, parse_uri_json_line, uri, ProcessUri, ProcessUriError};

#[cfg(feature = "python")]
mod py_ext {
    use super::*;
    use pyo3::exceptions::PyValueError;
    use pyo3::prelude::*;
    use serde_json::Value as JsonValue;

    #[pyfunction]
    fn parse_uri_json(input: &str) -> PyResult<String> {
        let parsed = ProcessUri::parse(input).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let out = serde_json::json!({
            "raw": parsed.raw,
            "scheme": parsed.scheme,
            "domain": parsed.domain,
            "action_or_resource": parsed.action_or_resource,
            "canonical_action": parsed.canonical_action(),
            "query_params": parsed.query_params,
            "is_urn": parsed.is_urn,
        });
        serde_json::to_string(&out).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[pyfunction]
    fn parse_uri_line_json(line: &str) -> PyResult<String> {
        let (parsed, payload) =
            parse_uri_json_line(line).map_err(|e| PyValueError::new_err(e.to_string()))?;
        let out = serde_json::json!({
            "uri": {
                "raw": parsed.raw,
                "scheme": parsed.scheme,
                "domain": parsed.domain,
                "action_or_resource": parsed.action_or_resource,
                "canonical_action": parsed.canonical_action(),
                "query_params": parsed.query_params,
                "is_urn": parsed.is_urn,
            },
            "payload": payload,
        });
        serde_json::to_string(&out).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[pyfunction]
    fn parse_jsonl_fast(payload: &str) -> PyResult<(String, bool)> {
        let (records, is_batch) =
            parse_jsonl_records(payload).map_err(PyValueError::new_err)?;
        let json_str =
            serde_json::to_string(&records).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok((json_str, is_batch))
    }

    #[pyfunction]
    fn dispatch_nl_json(input: &str) -> PyResult<String> {
        let (intent, uri) = dispatch_nl_to_uri(input);
        let out = serde_json::json!({
            "intent": format!("{:?}", intent),
            "uri": uri.canonical_action(),
            "scheme": uri.scheme,
            "domain": uri.domain,
            "action": uri.action_or_resource,
            "query_params": uri.query_params,
            "is_urn": uri.is_urn,
        });
        serde_json::to_string(&out).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[pyfunction]
    fn tokenize_calibrated_fast(text: &str) -> Vec<String> {
        tokenize_calibrated(text)
    }

    #[pyfunction]
    fn tokenize_decision_fast(text: &str) -> Vec<String> {
        tokenize_decision(text)
    }

    #[pyfunction]
    fn score_bindings_calibrated_json(
        request: &str,
        bindings_json: &str,
        temperature: f64,
    ) -> PyResult<String> {
        let bindings: JsonValue = serde_json::from_str(bindings_json)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let res = score_bindings_calibrated(request, &bindings, temperature);
        serde_json::to_string(&res).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[pyfunction]
    fn evaluate_decision_choice_json(
        query: &str,
        candidates_json: &str,
        temperature: f64,
        confidence_threshold: f64,
    ) -> PyResult<String> {
        let candidates: JsonValue = serde_json::from_str(candidates_json)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let res = evaluate_decision_choice(query, &candidates, temperature, confidence_threshold);
        serde_json::to_string(&res).map_err(|e| PyValueError::new_err(e.to_string()))
    }

    #[pymodule]
    pub fn process_uri_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(parse_uri_json, m)?)?;
        m.add_function(wrap_pyfunction!(parse_uri_line_json, m)?)?;
        m.add_function(wrap_pyfunction!(parse_jsonl_fast, m)?)?;
        m.add_function(wrap_pyfunction!(dispatch_nl_json, m)?)?;
        m.add_function(wrap_pyfunction!(tokenize_calibrated_fast, m)?)?;
        m.add_function(wrap_pyfunction!(tokenize_decision_fast, m)?)?;
        m.add_function(wrap_pyfunction!(score_bindings_calibrated_json, m)?)?;
        m.add_function(wrap_pyfunction!(evaluate_decision_choice_json, m)?)?;
        Ok(())
    }
}
