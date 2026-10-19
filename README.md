# process-uri (`if-uri/process-uri-rs`)

High-performance Rust crate (`process_uri_rs`) and Python package (`process_uri`) for URI-based operational systems (`Dockuri`, `Paxlet`, `Willman`, `Koru`, `URI Process Router`):

- **RFC 3986 Action URI & Resource URN Parser**: `parse_uri`, `parse_uri_line` (`koru://...`, `proc://...`, `willman://...`, `planfile://...`, `urn:...`).
- **System-1 Calibrated Discriminative URI Router Engine**: `tokenize_calibrated_fast`, `score_bindings_calibrated`, `evaluate_decision_choice` (accelerates `dockuri/uri-process-router` and `dockuri/willuri`).
- **NL-to-URI Dispatcher & GBNF Grammar Generator**: `dispatch_nl`, `export_gbnf`.
