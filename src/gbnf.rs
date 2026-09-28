use crate::registry::ProcessUriRegistry;

/// Export a GBNF (GGML BNF) grammar string constraining LLMs to only output registered URIs and JSON.
pub fn export_gbnf(registry: &ProcessUriRegistry) -> String {
    let actions = registry.actions();
    if actions.is_empty() {
        return "root ::= [a-z]+ \"://\" [a-zA-Z0-9_/-]+ (\" \" \"{\" [^}]* \"}\")?\n".to_string();
    }

    let mut uri_choices: Vec<String> = actions
        .keys()
        .map(|k| format!("\"{}\"", k))
        .collect();
    uri_choices.sort();
    let choices_str = uri_choices.join(" | ");

    format!(
        "root ::= action_line ( \"\\n\" action_line )*\n\
         action_line ::= uri ( \" \" json_payload )?\n\
         uri ::= {}\n\
         json_payload ::= \"{{\" [^}}\\n]* \"}}\"\n",
        choices_str
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_gbnf() {
        let mut reg = ProcessUriRegistry::new();
        reg.register("browser://navigate", "Navigate page", vec!["url"]);
        reg.register("koru://queue/claim", "Claim ticket", vec!["ticket_id"]);

        let gbnf = export_gbnf(&reg);
        assert!(gbnf.contains("\"browser://navigate\" | \"koru://queue/claim\""));
        assert!(gbnf.contains("json_payload ::="));
    }
}
