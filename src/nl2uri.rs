use crate::uri::ProcessUri;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum NlIntent {
    StopLoop,
    StartLoop,
    StatusLoop,
    QueueStatus,
    TaskNext,
    TaskClaim(String),
    TaskComplete(String),
    Unknown,
}

/// Normalizes Polish and English input: lowers case, replaces diacritics, strips punctuation.
pub fn normalize_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        let ch = match c {
            'Ą' | 'ą' => 'a',
            'Ć' | 'ć' => 'c',
            'Ę' | 'ę' => 'e',
            'Ł' | 'ł' => 'l',
            'Ń' | 'ń' => 'n',
            'Ó' | 'ó' => 'o',
            'Ś' | 'ś' => 's',
            'Ź' | 'ź' | 'Ż' | 'ż' => 'z',
            other => other.to_ascii_lowercase(),
        };
        if ch.is_alphanumeric() || ch.is_whitespace() || ch == '-' {
            out.push(ch);
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extracts a ticket ID like "PLF-123", "ticket-330", or digits.
pub fn extract_ticket_id(text: &str) -> Option<String> {
    for part in text.split_whitespace() {
        let clean = part.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        if clean.starts_with("plf-") || clean.starts_with("ticket-") {
            return Some(clean.to_uppercase());
        }
        if clean.chars().all(|c| c.is_ascii_digit()) && !clean.is_empty() {
            return Some(format!("ticket-{}", clean));
        }
    }
    None
}

/// Dispatches a natural language utterance to a structured NlIntent and corresponding ProcessUri.
pub fn dispatch_nl_to_uri(input: &str) -> (NlIntent, ProcessUri) {
    let norm = normalize_text(input);

    if norm.contains("stop") || norm.contains("zatrzymaj") || norm.contains("przerwij") {
        let uri = ProcessUri::parse("koru://loop/stop").unwrap();
        return (NlIntent::StopLoop, uri);
    }

    if norm.contains("start") || norm.contains("uruchom") || norm.contains("wznow") {
        let uri = ProcessUri::parse("koru://loop/start").unwrap();
        return (NlIntent::StartLoop, uri);
    }

    if norm.contains("status kolejki") || norm.contains("pokaz kolejke") || norm.contains("kolejka") {
        let uri = ProcessUri::parse("koru://queue/status").unwrap();
        return (NlIntent::QueueStatus, uri);
    }

    if norm.contains("status") || norm.contains("stan") {
        let uri = ProcessUri::parse("koru://loop/status").unwrap();
        return (NlIntent::StatusLoop, uri);
    }

    if norm.contains("zamknij") || norm.contains("zakoncz") || norm.contains("complete") {
        let tid = extract_ticket_id(&norm).unwrap_or_else(|| "current".to_string());
        let uri = ProcessUri::parse(&format!("koru://queue/task/complete?ticket_id={}", tid)).unwrap();
        return (NlIntent::TaskComplete(tid), uri);
    }

    if norm.contains("wez") || norm.contains("bierz") || norm.contains("claim") || norm.contains("wykonaj") {
        if let Some(tid) = extract_ticket_id(&norm) {
            let uri = ProcessUri::parse(&format!("koru://queue/task/claim?ticket_id={}", tid)).unwrap();
            return (NlIntent::TaskClaim(tid), uri);
        }
        let uri = ProcessUri::parse("koru://queue/task/next").unwrap();
        return (NlIntent::TaskNext, uri);
    }

    if norm.contains("nastepn") || norm.contains("kolejn") || norm.contains("next") || norm.contains("daj") {
        let uri = ProcessUri::parse("koru://queue/task/next").unwrap();
        return (NlIntent::TaskNext, uri);
    }

    let uri = ProcessUri {
        raw: "koru://unknown".to_string(),
        scheme: "koru".to_string(),
        domain: "unknown".to_string(),
        action_or_resource: String::new(),
        query_params: BTreeMap::new(),
        is_urn: false,
    };
    (NlIntent::Unknown, uri)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dispatch_stop() {
        let (intent, uri) = dispatch_nl_to_uri("Zatrzymaj natychmiast pętlę");
        assert_eq!(intent, NlIntent::StopLoop);
        assert_eq!(uri.canonical_action(), "koru://loop/stop");
    }

    #[test]
    fn test_dispatch_claim_ticket() {
        let (intent, uri) = dispatch_nl_to_uri("Weź proszę bilet 338");
        assert_eq!(intent, NlIntent::TaskClaim("ticket-338".to_string()));
        assert_eq!(uri.canonical_action(), "koru://queue/task/claim");
        assert_eq!(uri.query_params.get("ticket_id").unwrap(), "ticket-338");
    }

    #[test]
    fn test_dispatch_complete() {
        let (intent, uri) = dispatch_nl_to_uri("Zakończ PLF-100");
        assert_eq!(intent, NlIntent::TaskComplete("PLF-100".to_string()));
        assert_eq!(uri.canonical_action(), "koru://queue/task/complete");
        assert_eq!(uri.query_params.get("ticket_id").unwrap(), "PLF-100");
    }

    #[test]
    fn test_dispatch_next() {
        let (intent, uri) = dispatch_nl_to_uri("Daj następne zadanie");
        assert_eq!(intent, NlIntent::TaskNext);
        assert_eq!(uri.canonical_action(), "koru://queue/task/next");
    }
}
