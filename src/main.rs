use std::env;
use std::process;

use process_uri::{dispatch_nl_to_uri, export_gbnf, parse_uri_json_line, ProcessUriRegistry};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: process-uri <command> [args...]");
        eprintln!("Commands:");
        eprintln!("  parse <uri>       Parse a Process URI and print JSON");
        eprintln!("  nl2uri <prompt>   Dispatch natural language to URI intent");
        eprintln!("  gbnf              Print default GBNF grammar");
        process::exit(1);
    }

    match args[1].as_str() {
        "parse" => {
            if args.len() < 3 {
                eprintln!("Error: missing URI argument");
                process::exit(1);
            }
            match parse_uri_json_line(&args[2]) {
                Ok((_uri, payload)) => println!("{}", serde_json::to_string_pretty(&payload).unwrap()),
                Err(err) => {
                    eprintln!("Error: {}", err);
                    process::exit(2);
                }
            }
        }
        "nl2uri" => {
            if args.len() < 3 {
                eprintln!("Error: missing text argument");
                process::exit(1);
            }
            let query = args[2..].join(" ");
            let (intent, uri) = dispatch_nl_to_uri(&query);
            let result = serde_json::json!({
                "intent": format!("{:?}", intent),
                "uri": uri.canonical_action(),
                "scheme": uri.scheme,
                "domain": uri.domain,
                "action": uri.action_or_resource,
                "query_params": uri.query_params,
                "is_urn": uri.is_urn,
            });
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        "gbnf" => {
            let registry = ProcessUriRegistry::default();
            println!("{}", export_gbnf(&registry));
        }
        unknown => {
            eprintln!("Unknown command: {}", unknown);
            process::exit(1);
        }
    }
}
