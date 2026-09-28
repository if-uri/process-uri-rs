use std::collections::BTreeMap;
use std::fmt;
use url::Url;

#[derive(Debug, PartialEq, Eq)]
pub enum ProcessUriError {
    EmptyUri,
    MissingScheme(String),
    MissingDomain(String),
    MalformedUrn(String),
    UrlParse(String),
}

impl fmt::Display for ProcessUriError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyUri => write!(f, "Empty URI string"),
            Self::MissingScheme(s) => write!(f, "Missing URI scheme in '{}'", s),
            Self::MissingDomain(s) => write!(f, "Missing domain/host in URI '{}'", s),
            Self::MalformedUrn(s) => write!(f, "Malformed URN: '{}', expected format urn:<domain>:<type>:<id>", s),
            Self::UrlParse(e) => write!(f, "URI parse error: {}", e),
        }
    }
}

impl std::error::Error for ProcessUriError {}

/// Represents a parsed and validated Action URI or Resource URN.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProcessUri {
    pub raw: String,
    pub scheme: String,
    pub domain: String,
    pub action_or_resource: String,
    pub query_params: BTreeMap<String, String>,
    pub is_urn: bool,
}

impl ProcessUri {
    /// Parse an action URI (scheme://domain/action?query) or URN (urn:domain:type:id).
    pub fn parse(input: &str) -> Result<Self, ProcessUriError> {
        let s = input.trim();
        if s.is_empty() {
            return Err(ProcessUriError::EmptyUri);
        }

        if s.starts_with("urn:") {
            let parts: Vec<&str> = s.split(':').collect();
            if parts.len() < 4 {
                return Err(ProcessUriError::MalformedUrn(s.to_string()));
            }
            let domain = parts[1].to_string();
            let resource = parts[2..].join(":");
            return Ok(Self {
                raw: s.to_string(),
                scheme: "urn".to_string(),
                domain,
                action_or_resource: resource,
                query_params: BTreeMap::new(),
                is_urn: true,
            });
        }

        let parsed = Url::parse(s).map_err(|e| ProcessUriError::UrlParse(e.to_string()))?;
        let scheme = parsed.scheme().to_string();
        if scheme.is_empty() {
            return Err(ProcessUriError::MissingScheme(s.to_string()));
        }

        let domain = match parsed.host_str() {
            Some(h) if !h.is_empty() => h.to_string(),
            _ => return Err(ProcessUriError::MissingDomain(s.to_string())),
        };

        let action = parsed.path().trim_start_matches('/').to_string();

        let mut query_params = BTreeMap::new();
        for (k, v) in parsed.query_pairs() {
            query_params.insert(k.to_string(), v.to_string());
        }

        Ok(Self {
            raw: s.to_string(),
            scheme,
            domain,
            action_or_resource: action,
            query_params,
            is_urn: false,
        })
    }

    /// Return scheme://domain/action without query parameters.
    pub fn canonical_action(&self) -> String {
        if self.is_urn {
            self.raw.clone()
        } else {
            format!("{}://{}/{}", self.scheme, self.domain, self.action_or_resource)
        }
    }
}

/// Parse a line in the format: 'yaml: uri {json}', 'uri {json}', or '- action: uri {json}'.
pub fn parse_uri_json_line(line: &str) -> Result<(ProcessUri, serde_json::Value), ProcessUriError> {
    let mut s = line.trim();
    if let Some(stripped) = s.strip_prefix("- ") {
        s = stripped.trim();
    }
    if let Some(stripped) = s.strip_prefix("action:") {
        s = stripped.trim();
    }
    if let Some(stripped) = s.strip_prefix("uri:") {
        s = stripped.trim();
    }

    let (uri_part, json_part) = match s.split_once(' ') {
        Some((u, j)) => (u.trim(), Some(j.trim())),
        None => (s, None),
    };

    let uri = ProcessUri::parse(uri_part)?;
    let payload = match json_part {
        Some(j) if !j.is_empty() => serde_json::from_str(j).unwrap_or(serde_json::Value::Null),
        _ => serde_json::Value::Null,
    };

    Ok((uri, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_action_uri() {
        let uri = ProcessUri::parse("koru://queue/task/claim?ticket_id=330&dry_run=true").unwrap();
        assert_eq!(uri.scheme, "koru");
        assert_eq!(uri.domain, "queue");
        assert_eq!(uri.action_or_resource, "task/claim");
        assert_eq!(uri.canonical_action(), "koru://queue/task/claim");
        assert_eq!(uri.query_params.get("ticket_id").unwrap(), "330");
        assert_eq!(uri.query_params.get("dry_run").unwrap(), "true");
        assert!(!uri.is_urn);
    }

    #[test]
    fn test_parse_urn() {
        let urn = ProcessUri::parse("urn:koru:ticket:330").unwrap();
        assert_eq!(urn.scheme, "urn");
        assert_eq!(urn.domain, "koru");
        assert_eq!(urn.action_or_resource, "ticket:330");
        assert_eq!(urn.canonical_action(), "urn:koru:ticket:330");
        assert!(urn.is_urn);
    }

    #[test]
    fn test_parse_line_with_json() {
        let line = "uri: browser://navigate {\"url\": \"https://example.com\"}";
        let (uri, payload) = parse_uri_json_line(line).unwrap();
        assert_eq!(uri.canonical_action(), "browser://navigate/");
        assert_eq!(payload["url"], "https://example.com");
    }
}
