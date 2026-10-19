//! High-performance System-1 Calibrated Discriminative URI Decision Engine.
//! Accelerates `dockuri/uri-process-router` (`LocalCalibratedBackend` & `DecisionEngine`)
//! and `dockuri/willuri` domain classification in native Rust.

use serde_json::{json, Value as JsonValue};
use std::collections::{BTreeMap, HashSet};

/// Normalize Polish diacritics, phonetic variants (`cgh`->`ch`, `gh`->`h`), and tokenize with 3/4-char prefix subgrams.
pub fn tokenize_calibrated(text: &str) -> Vec<String> {
    let lower = text
        .to_lowercase()
        .replace("cgh", "ch")
        .replace("gh", "h");
    let mut norm = String::with_capacity(lower.len());
    for c in lower.chars() {
        let mapped = match c {
            'ł' => 'l',
            'ó' => 'o',
            'ę' => 'e',
            'ą' => 'a',
            'ś' => 's',
            'ć' => 'c',
            'ż' | 'ź' => 'z',
            'ń' => 'n',
            other if other.is_alphanumeric() || other.is_whitespace() || other == '.' || other == '-' || other == '_' => other,
            _ => ' ',
        };
        norm.push(mapped);
    }

    let raw_tokens: Vec<&str> = norm
        .split_whitespace()
        .filter(|t| t.chars().count() > 1)
        .collect();

    let mut tokens: Vec<String> = Vec::with_capacity(raw_tokens.len() * 3);
    for t in raw_tokens {
        tokens.push(t.to_string());
        if t.contains('.') {
            for part in t.split('.') {
                if part.chars().count() > 1 {
                    tokens.push(part.to_string());
                }
            }
        }
    }

    let mut sub4: Vec<String> = Vec::new();
    let mut sub3: Vec<String> = Vec::new();
    for t in &tokens {
        let chars: Vec<char> = t.chars().collect();
        if chars.len() >= 4 {
            sub4.push(chars[..4].iter().collect());
        }
        if chars.len() >= 3 {
            sub3.push(chars[..3].iter().collect());
        }
    }
    tokens.extend(sub4);
    tokens.extend(sub3);
    tokens
}

/// Tokenize for `DecisionEngine.choice` (Unicode lower + 4-char subgrams).
pub fn tokenize_decision(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut norm = String::with_capacity(lower.len());
    for c in lower.chars() {
        if c.is_alphanumeric() || c.is_whitespace() || c == '.' || c == '-' || c == '_' {
            norm.push(c);
        } else {
            norm.push(' ');
        }
    }
    let tokens: Vec<String> = norm
        .split_whitespace()
        .filter(|t| t.chars().count() > 1)
        .map(|s| s.to_string())
        .collect();
    let mut subgrams: Vec<String> = Vec::new();
    for t in &tokens {
        let chars: Vec<char> = t.chars().collect();
        if chars.len() >= 4 {
            subgrams.push(chars[..4].iter().collect());
        }
    }
    let mut out = tokens;
    out.extend(subgrams);
    out
}

fn has_negated_token(req_lower: &str, token: &str) -> bool {
    if token.chars().count() < 4 {
        return false;
    }
    for prefix in ["nie ", "don't ", "do not ", "nicht "] {
        let pat = format!("{}{}", prefix, token);
        if let Some(pos) = req_lower.find(&pat) {
            let prev_ok = pos == 0
                || !req_lower[..pos]
                    .chars()
                    .last()
                    .map(|c| c.is_alphanumeric())
                    .unwrap_or(false);
            if prev_ok {
                return true;
            }
        }
    }
    false
}

/// Score a single candidate contract using the exact `LocalCalibratedBackend._score_candidate` rules.
pub fn score_candidate_calibrated(request: &str, req_tokens: &HashSet<String>, contract: &JsonValue) -> f64 {
    let req_lower = request.to_lowercase();
    let desc = contract
        .get("desc")
        .and_then(|v| v.as_str())
        .or_else(|| contract.get("description").and_then(|v| v.as_str()))
        .unwrap_or("");
    let name = contract.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let uri = contract.get("uri").and_then(|v| v.as_str()).unwrap_or("");

    let cand_identity_tokens = tokenize_calibrated(&format!("{} {} {}", name, uri, desc));
    for t in &cand_identity_tokens {
        if has_negated_token(&req_lower, t) {
            return -1.0;
        }
    }

    if let Some(not_for) = contract.get("not_for").and_then(|v| v.as_array()) {
        for item in not_for {
            let Some(not_str) = item.as_str() else {
                continue;
            };
            let not_lower = not_str.to_lowercase();
            if not_lower.contains("wyjasnienie") || not_lower.contains("wyjaśnienie") {
                if ["jak dziala", "jak działa", "wyjasnij", "wyjaśnij", "co to jest"]
                    .iter()
                    .any(|w| req_lower.contains(w))
                {
                    return -0.80;
                }
            }
            if not_lower.contains("sprawdzenie")
                && (req_lower.contains("sprawdz") || req_lower.contains("status"))
            {
                if req_lower.contains("nie restartuj") || req_lower.contains("bez restartu") {
                    return -0.80;
                }
            }
            if not_lower.contains("log")
                && (req_lower.contains("logi")
                    || req_lower.contains("logów")
                    || req_lower.contains("dziennik"))
            {
                return -0.80;
            }
            if not_lower.contains("wpisywanie tekstu")
                && (req_lower.contains("wpisz") || req_lower.contains("napisz"))
            {
                return -0.80;
            }
        }
    }

    let mut tags_joined = String::new();
    if let Some(tags) = contract.get("tags").and_then(|v| v.as_array()) {
        for t in tags.iter().filter_map(|v| v.as_str()) {
            if !tags_joined.is_empty() {
                tags_joined.push(' ');
            }
            tags_joined.push_str(t);
        }
    }

    let cand_text = format!("{} {} {}", desc, uri, tags_joined);
    let cand_tokens: HashSet<String> = tokenize_calibrated(&cand_text).into_iter().collect();
    if cand_tokens.is_empty() || req_tokens.is_empty() {
        return 0.0;
    }

    let intersection_count = req_tokens.intersection(&cand_tokens).count() as f64;
    let denom = ((req_tokens.len() * cand_tokens.len()) as f64).sqrt();
    let mut base_score = if denom > 0.0 {
        intersection_count / denom
    } else {
        0.0
    };

    if let Some(examples) = contract.get("examples").and_then(|v| v.as_array()) {
        for ex in examples {
            let ex_utt = if let Some(obj) = ex.as_object() {
                obj.get("utterance")
                    .and_then(|v| v.as_str())
                    .or_else(|| obj.get("query").and_then(|v| v.as_str()))
                    .unwrap_or("")
            } else {
                ex.as_str().unwrap_or("")
            };
            if !ex_utt.is_empty() {
                let ex_low = ex_utt.to_lowercase();
                if ex_low.trim() == req_lower.trim() {
                    return 1.0;
                }
                let ex_tokens: HashSet<String> = tokenize_calibrated(ex_utt).into_iter().collect();
                let ex_sim =
                    (req_tokens.intersection(&ex_tokens).count() as f64) / (req_tokens.len().max(1) as f64);
                if ex_sim > 0.65 {
                    base_score += 0.50;
                } else if req_lower.contains(&ex_low) || ex_low.contains(&req_lower) {
                    base_score += 0.40;
                }
            }
        }
    }

    let has_param = |p: &str| -> bool {
        contract
            .get("parameters")
            .and_then(|v| v.as_object())
            .map(|m| m.contains_key(p))
            .unwrap_or(false)
    };

    if has_param("url") && (request.contains("http://") || request.contains("https://")) {
        base_score += 0.35;
    }
    if has_param("layout")
        && [
            "uklad", "układ", "grid", "cols", "rows", "stage", "carousel", "siatk", "kolumn",
            "karuzel",
        ]
        .iter()
        .any(|w| req_lower.contains(w))
    {
        base_score += 0.30;
    }
    if uri.ends_with("/window/focus/v1")
        && ["fokus", "focus", "przelacz", "przełącz"]
            .iter()
            .any(|w| req_lower.contains(w))
    {
        base_score += 0.30;
    }
    if uri.ends_with("/input/type/v1")
        && ["wpisz", "napisz", "type"]
            .iter()
            .any(|w| req_lower.contains(w))
    {
        base_score += 0.35;
    }
    if uri.ends_with("/input/key/v1")
        && [
            "klawisz", "key", "nacisnij", "naciśnij", "wcisnij", "wciśnij", "enter", "return",
            "escape",
        ]
        .iter()
        .any(|w| req_lower.contains(w))
    {
        base_score += 0.35;
    }
    if uri.ends_with("/window/zoom/v1")
        && [
            "powieksz",
            "powiększ",
            "zoom",
            "pełny ekran",
            "pelny ekran",
            "fullscreen",
            "cały ekran",
            "caly ekran",
            "maksymalizuj",
            "rozwin",
            "rozwiń",
        ]
        .iter()
        .any(|w| req_lower.contains(w))
    {
        base_score += 0.40;
    }
    if uri.ends_with("/window/swap/v1")
        && ["zamien", "zamień", "swap"]
            .iter()
            .any(|w| req_lower.contains(w))
    {
        base_score += 0.35;
    }
    if uri.ends_with("/svg-to-png/v1") && req_lower.contains(".svg") && req_lower.contains(".png") {
        if let (Some(s_idx), Some(p_idx)) = (req_lower.find(".svg"), req_lower.find(".png")) {
            if s_idx < p_idx {
                base_score += 0.45;
            }
        }
    }
    if uri.ends_with("/png-to-svg/v1") && req_lower.contains(".png") && req_lower.contains(".svg") {
        if let (Some(p_idx), Some(s_idx)) = (req_lower.find(".png"), req_lower.find(".svg")) {
            if p_idx < s_idx {
                base_score += 0.45;
            }
        }
    }
    if uri.ends_with("/service/restart/v1")
        && [
            "restart",
            "restartuj",
            "ponownie",
            "reboot",
            "uruchom ponownie",
        ]
        .iter()
        .any(|w| req_lower.contains(w))
    {
        base_score += 0.40;
    }
    if uri.ends_with("/service/status/v1")
        && ["status", "stan", "sprawdz", "sprawdź", "healthcheck"]
            .iter()
            .any(|w| req_lower.contains(w))
    {
        base_score += 0.40;
    }
    if uri.ends_with("/media/play/v1")
        && [
            "piosenk", "muzyk", "youtube", "zagraj", "odtworz", "odtwórz", "film", "wideo",
            "video", "graj",
        ]
        .iter()
        .any(|w| req_lower.contains(w))
    {
        base_score += 0.45;
    }
    if uri.ends_with("/option/set/v1")
        && [
            "opcj",
            "option",
            "preferencj",
            "llm-setup",
            "page-mode",
            "palette-layout",
            "tryb konfiguracji",
            "układ poleceń",
            "uklad polecen",
        ]
        .iter()
        .any(|w| req_lower.contains(w))
    {
        base_score += 0.45;
    }

    base_score.max(0.0)
}

/// Score all candidate bindings and compute Softmax probabilities in native Rust.
pub fn score_bindings_calibrated(
    request: &str,
    bindings: &JsonValue,
    temperature: f64,
) -> JsonValue {
    let clean_req = request.trim();
    let req_tokens: HashSet<String> = tokenize_calibrated(clean_req).into_iter().collect();
    let mut raw_scores: BTreeMap<String, f64> = BTreeMap::new();

    if let Some(map) = bindings.as_object() {
        for (option_id, contract) in map {
            let score = score_candidate_calibrated(clean_req, &req_tokens, contract);
            raw_scores.insert(option_id.clone(), score);
        }
    }

    raw_scores.insert("NO_MATCH".to_string(), 0.25);
    raw_scores.insert("CLARIFY".to_string(), 0.05);
    raw_scores.insert("PLAN_REQUIRED".to_string(), 0.05);

    let max_score = raw_scores
        .values()
        .copied()
        .fold(f64::NEG_INFINITY, f64::max);
    let temp = if temperature > 0.0 { temperature } else { 0.18 };

    let mut exp_scores: BTreeMap<String, f64> = BTreeMap::new();
    let mut sum_exp = 0.0;
    for (k, v) in &raw_scores {
        let e = ((*v - max_score) / temp).exp();
        exp_scores.insert(k.clone(), e);
        sum_exp += e;
    }

    let mut probs: BTreeMap<String, f64> = BTreeMap::new();
    let mut best_id = String::from("NO_MATCH");
    let mut best_prob = -1.0;

    for (k, e) in &exp_scores {
        let p = if sum_exp > 0.0 { *e / sum_exp } else { 0.0 };
        probs.insert(k.clone(), p);
        if p > best_prob {
            best_prob = p;
            best_id = k.clone();
        }
    }

    json!({
        "best_id": best_id,
        "best_prob": best_prob,
        "best_raw_score": raw_scores.get(&best_id).copied().unwrap_or(0.0),
        "raw_scores": raw_scores,
        "probabilities": probs,
    })
}

/// Evaluate `DecisionEngine.choice` in native Rust.
pub fn evaluate_decision_choice(
    query: &str,
    candidates: &JsonValue,
    temperature: f64,
    confidence_threshold: f64,
) -> JsonValue {
    let Some(cands) = candidates.as_array() else {
        return json!({
            "selected": null,
            "confidence": 0.0,
            "scores": {},
            "abstain": true,
            "reason": "No candidate operations available.",
        });
    };
    if cands.is_empty() {
        return json!({
            "selected": null,
            "confidence": 0.0,
            "scores": {},
            "abstain": true,
            "reason": "No candidate operations available.",
        });
    }

    let query_tokens: HashSet<String> = tokenize_decision(query).into_iter().collect();
    if query_tokens.is_empty() {
        return json!({
            "selected": null,
            "confidence": 0.0,
            "scores": {},
            "abstain": true,
            "reason": "Empty query tokens.",
        });
    }

    let q_lower = query.to_lowercase();
    let mut raw_scores: Vec<(String, f64)> = Vec::with_capacity(cands.len());

    for cand in cands {
        let Some(uri) = cand.get("uri").and_then(|v| v.as_str()) else {
            continue;
        };
        let desc = cand.get("desc").and_then(|v| v.as_str()).unwrap_or("");
        let mut corpus = format!("{} {}", desc, uri);
        if let Some(tags) = cand.get("tags").and_then(|v| v.as_array()) {
            for t in tags.iter().filter_map(|v| v.as_str()) {
                corpus.push(' ');
                corpus.push_str(t);
            }
        }
        if let Some(examples) = cand.get("examples").and_then(|v| v.as_array()) {
            for ex in examples {
                let ex_text = if let Some(obj) = ex.as_object() {
                    obj.get("utterance")
                        .and_then(|v| v.as_str())
                        .or_else(|| obj.get("query").and_then(|v| v.as_str()))
                        .unwrap_or("")
                } else {
                    ex.as_str().unwrap_or("")
                };
                if !ex_text.is_empty() {
                    corpus.push(' ');
                    corpus.push_str(ex_text);
                    corpus.push_str(ex_text);
                }
            }
        }

        let cand_tokens: HashSet<String> = tokenize_decision(&corpus).into_iter().collect();
        if cand_tokens.is_empty() {
            raw_scores.push((uri.to_string(), 0.0));
            continue;
        }

        let inter = query_tokens.intersection(&cand_tokens).count() as f64;
        let mut overlap =
            inter / ((query_tokens.len() as f64).sqrt() * (cand_tokens.len() as f64).sqrt());

        if let Some(examples) = cand.get("examples").and_then(|v| v.as_array()) {
            for ex in examples {
                let ex_text = if let Some(obj) = ex.as_object() {
                    obj.get("utterance")
                        .and_then(|v| v.as_str())
                        .or_else(|| obj.get("query").and_then(|v| v.as_str()))
                        .unwrap_or("")
                } else {
                    ex.as_str().unwrap_or("")
                };
                if !ex_text.is_empty() {
                    let ex_low = ex_text.to_lowercase();
                    if q_lower.contains(&ex_low) || ex_low.contains(&q_lower) {
                        overlap += 0.50;
                    }
                }
            }
        }

        if let Some(not_for) = cand.get("not_for").and_then(|v| v.as_array()) {
            for nf in not_for {
                let nf_str = nf.as_str().unwrap_or("");
                let nf_tokens: HashSet<String> = tokenize_decision(nf_str).into_iter().collect();
                let nf_overlap = query_tokens.intersection(&nf_tokens).count();
                if nf_overlap >= 2 || (nf_tokens.len() <= 3 && nf_overlap >= 1) {
                    overlap -= 0.60;
                }
            }
        }

        let uri_parts: HashSet<String> = uri
            .to_lowercase()
            .split(|c| c == '/' || c == ':' || c == '-' || c == '_')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
        let uri_matches = query_tokens.intersection(&uri_parts).count() as f64;
        overlap += uri_matches * 0.15;

        raw_scores.push((uri.to_string(), overlap.max(0.0)));
    }

    let max_raw = raw_scores.iter().map(|(_, s)| *s).fold(0.0_f64, f64::max);
    if max_raw == 0.0 {
        let zero_map: BTreeMap<String, f64> =
            raw_scores.into_iter().map(|(u, _)| (u, 0.0)).collect();
        return json!({
            "selected": null,
            "confidence": 0.0,
            "scores": zero_map,
            "abstain": true,
            "reason": "Zero semantic overlap with registry operations.",
        });
    }

    let temp = if temperature > 0.0 { temperature } else { 0.25 };
    let mut exp_scores: Vec<(String, f64, f64)> = Vec::with_capacity(raw_scores.len());
    let mut sum_exp = 0.0;
    for (u, s) in raw_scores {
        let e = ((s - max_raw) / temp).exp();
        sum_exp += e;
        exp_scores.push((u, s, e));
    }

    let mut calibrated: BTreeMap<String, f64> = BTreeMap::new();
    let mut best_uri = String::new();
    let mut best_conf = -1.0;
    let mut best_raw = 0.0;

    for (u, raw, e) in exp_scores {
        let c = e / sum_exp;
        if c > best_conf {
            best_conf = c;
            best_uri = u.clone();
            best_raw = raw;
        }
        calibrated.insert(u, c);
    }

    if best_raw < 0.18 || best_conf < confidence_threshold {
        return json!({
            "selected": best_uri,
            "confidence": best_conf,
            "scores": calibrated,
            "abstain": true,
            "reason": format!(
                "Confidence {:.2} or semantic match below threshold {:.2}.",
                best_conf, confidence_threshold
            ),
        });
    }

    json!({
        "selected": best_uri,
        "confidence": best_conf,
        "scores": calibrated,
        "abstain": false,
        "reason": format!("Calibrated match with confidence {:.2}", best_conf),
    })
}
