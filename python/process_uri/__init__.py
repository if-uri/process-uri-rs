"""Python bindings for `if-uri/process-uri-rs`: Rust Process URI parser and System-1 calibrated router."""

from __future__ import annotations

import json
from pathlib import Path
import sys
from typing import Any, Iterable

try:
    from .process_uri_rs import (
        dispatch_nl_json as _rs_dispatch_nl_json,
        evaluate_decision_choice_json as _rs_evaluate_decision_choice_json,
        parse_jsonl_fast as _rs_parse_jsonl_fast,
        parse_uri_json as _rs_parse_uri_json,
        parse_uri_line_json as _rs_parse_uri_line_json,
        score_bindings_calibrated_json as _rs_score_bindings_calibrated_json,
        tokenize_calibrated_fast as _rs_tokenize_calibrated_fast,
        tokenize_decision_fast as _rs_tokenize_decision_fast,
    )

    HAS_RUST_PROCESS_URI = True
except ImportError:
    HAS_RUST_PROCESS_URI = False


def parse_uri(uri_str: str) -> dict[str, Any]:
    """Parse an RFC 3986 Action URI (`scheme://domain/action?k=v`) or Resource URN (`urn:domain:type:id`)."""
    if HAS_RUST_PROCESS_URI:
        return json.loads(_rs_parse_uri_json(uri_str))
    raise RuntimeError("process_uri_rs native extension is not available")


def parse_uri_line(line: str) -> tuple[dict[str, Any], Any]:
    """Parse a gateway/action line (`uri: scheme://... {json}`) into `(uri_dict, payload)`."""
    if HAS_RUST_PROCESS_URI:
        data = json.loads(_rs_parse_uri_line_json(line))
        return data["uri"], data["payload"]
    raise RuntimeError("process_uri_rs native extension is not available")


def parse_jsonl_fast(payload: str) -> tuple[list[dict[str, Any]], bool]:
    """Parse a single JSON object or multiline JSONL string in native Rust."""
    if HAS_RUST_PROCESS_URI:
        raw_json, is_batch = _rs_parse_jsonl_fast(payload)
        return json.loads(raw_json), bool(is_batch)
    raise RuntimeError("process_uri_rs native extension is not available")


def dispatch_nl(text: str) -> dict[str, Any]:
    """Deterministically dispatch a natural-language utterance (PL/EN) to a `koru://` process URI."""
    if HAS_RUST_PROCESS_URI:
        return json.loads(_rs_dispatch_nl_json(text))
    raise RuntimeError("process_uri_rs native extension is not available")


def tokenize_calibrated_fast(text: str) -> list[str]:
    """Tokenize and normalize PL/EN text with 3/4-char prefix subgrams for calibrated URI routing."""
    if HAS_RUST_PROCESS_URI:
        return _rs_tokenize_calibrated_fast(text)
    raise RuntimeError("process_uri_rs native extension is not available")


def tokenize_decision_fast(text: str) -> list[str]:
    """Tokenize text for System-1 DecisionEngine choice scoring."""
    if HAS_RUST_PROCESS_URI:
        return _rs_tokenize_decision_fast(text)
    raise RuntimeError("process_uri_rs native extension is not available")


def score_bindings_calibrated(
    request: str,
    bindings: dict[str, dict[str, Any]],
    temperature: float = 0.18,
) -> dict[str, Any]:
    """Score candidate URI contracts and compute Softmax probabilities in native Rust."""
    if HAS_RUST_PROCESS_URI:
        raw = _rs_score_bindings_calibrated_json(
            request, json.dumps(bindings, ensure_ascii=False), float(temperature)
        )
        return json.loads(raw)
    raise RuntimeError("process_uri_rs native extension is not available")


def evaluate_decision_choice(
    query: str,
    candidates: list[dict[str, Any]],
    temperature: float = 0.25,
    confidence_threshold: float = 0.55,
) -> dict[str, Any]:
    """Evaluate System-1 discriminative URI choice across candidates in native Rust."""
    if HAS_RUST_PROCESS_URI:
        raw = _rs_evaluate_decision_choice_json(
            query,
            json.dumps(list(candidates), ensure_ascii=False),
            float(temperature),
            float(confidence_threshold),
        )
        return json.loads(raw)
    raise RuntimeError("process_uri_rs native extension is not available")


def _load_dockuri_uri() -> Any:
    try:
        import dockuri_uri
        return dockuri_uri
    except ImportError:
        cand = Path.home() / "github" / "dockuri" / "dockuri" / "python"
        if cand.is_dir() and str(cand) not in sys.path:
            sys.path.insert(0, str(cand))
        import dockuri_uri
        return dockuri_uri


def uri(address: str, payload: Any = None, **kwargs: Any) -> Any:
    """Universal `uri('<full-uri>', [payload_or_jsonl])` caller backed by `dockuri_uri`."""
    return _load_dockuri_uri().uri(address, payload, **kwargs)


def uri_jsonl(address: str, lines: Iterable[str | dict[str, Any]], **kwargs: Any) -> list[dict[str, Any]]:
    """Stream JSONL lines or dicts into a `dockuri` URI process."""
    return _load_dockuri_uri().uri_jsonl(address, lines, **kwargs)


__all__ = [
    "HAS_RUST_PROCESS_URI",
    "dispatch_nl",
    "evaluate_decision_choice",
    "parse_jsonl_fast",
    "parse_uri",
    "parse_uri_line",
    "score_bindings_calibrated",
    "tokenize_calibrated_fast",
    "tokenize_decision_fast",
    "uri",
    "uri_jsonl",
]

