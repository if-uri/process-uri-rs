from process_uri import (
    HAS_RUST_PROCESS_URI,
    dispatch_nl,
    evaluate_decision_choice,
    parse_uri,
    parse_uri_line,
    score_bindings_calibrated,
    tokenize_calibrated_fast,
)


def test_has_rust_process_uri():
    assert HAS_RUST_PROCESS_URI is True


def test_parse_uri_and_urn():
    parsed = parse_uri("koru://queue/task/claim?ticket_id=330&dry_run=true")
    assert parsed["scheme"] == "koru"
    assert parsed["domain"] == "queue"
    assert parsed["canonical_action"] == "koru://queue/task/claim"
    assert parsed["query_params"]["ticket_id"] == "330"
    assert parsed["is_urn"] is False

    urn = parse_uri("urn:koru:ticket:330")
    assert urn["is_urn"] is True
    assert urn["domain"] == "koru"
    assert urn["action_or_resource"] == "ticket:330"

    uri_dict, payload = parse_uri_line('uri: browser://navigate {"url": "https://example.com"}')
    assert uri_dict["scheme"] == "browser"
    assert payload["url"] == "https://example.com"


def test_dispatch_nl():
    res = dispatch_nl("Weź proszę bilet 338")
    assert res["uri"] == "koru://queue/task/claim"
    assert res["query_params"]["ticket_id"] == "ticket-338"


def test_calibrated_router_and_decision_choice():
    tokens = tokenize_calibrated_fast("Przełącz układ na siatkę (grid)")
    assert "grid" in tokens
    assert "siatke" in tokens

    bindings = {
        "OPT_1": {
            "uri": "proc://willmux/window/layout/v1",
            "desc": "Zmienia układ okien na grid, cols, rows",
            "tags": ["window", "layout", "grid"],
            "parameters": {"layout": {"type": "string"}},
            "examples": [{"utterance": "Zmień układ na siatkę"}],
        },
        "OPT_2": {
            "uri": "proc://ops/service/restart/v1",
            "desc": "Restartuje usługę",
            "tags": ["service", "restart"],
            "parameters": {"service": {"type": "string"}},
        },
    }
    scored = score_bindings_calibrated("Zmień układ okien na grid", bindings, temperature=0.18)
    assert scored["best_id"] == "OPT_1"
    assert scored["best_prob"] > 0.7

    choice = evaluate_decision_choice(
        "Zmień układ okien na grid",
        list(bindings.values()),
        temperature=0.25,
        confidence_threshold=0.55,
    )
    assert choice["selected"] == "proc://willmux/window/layout/v1"
    assert choice["abstain"] is False
