use std::io::{self, BufRead, Write};

use agtravelcraft_core::bus::Command;
use agtravelcraft_core::skills::{list_skills, read_skill};
use agtravelcraft_core::store::TravelStore;
use serde_json::{Value, json};

pub const PROTOCOL_VERSION: &str = "2026-07-28";

pub fn run_stdio(store: &TravelStore) -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let raw = line?;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(resp) = handle_json_line(store, trimmed) {
            writeln!(stdout, "{resp}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

pub fn handle_json_line(store: &TravelStore, line: &str) -> Option<String> {
    let req: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return Some(
                json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": { "code": -32700, "message": format!("parse error: {e}") }
                })
                .to_string(),
            );
        }
    };
    handle_request(store, &req).map(|v| v.to_string())
}

pub fn handle_request(store: &TravelStore, req: &Value) -> Option<Value> {
    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or_else(|| json!({}));

    if method == "notifications/initialized" || id.is_none() {
        return None;
    }
    let id_val = id.unwrap_or(Value::Null);

    match method {
        "initialize" => Some(ok_resp(
            id_val,
            json!({
                "protocolVersion": PROTOCOL_VERSION,
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "subscribe": false, "listChanged": false }
                },
                "serverInfo": {
                    "name": "agtravelcraft-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }),
        )),
        "ping" => Some(ok_resp(id_val, json!({}))),
        "tools/list" => Some(ok_resp(id_val, json!({ "tools": tool_catalog() }))),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));
            Some(ok_resp(id_val, call_tool(store, name, &args)))
        }
        "resources/list" => {
            let resources: Vec<Value> = list_skills()
                .into_iter()
                .map(|s| {
                    json!({
                        "uri": s.uri,
                        "name": s.name,
                        "description": s.description,
                        "mimeType": "text/markdown"
                    })
                })
                .collect();
            Some(ok_resp(id_val, json!({ "resources": resources })))
        }
        "resources/read" => {
            let uri = params.get("uri").and_then(Value::as_str).unwrap_or("");
            match read_skill(uri) {
                Some(skill) => Some(ok_resp(
                    id_val,
                    json!({
                        "contents": [{
                            "uri": skill.uri,
                            "mimeType": "text/markdown",
                            "text": skill.content
                        }]
                    }),
                )),
                None => Some(err_resp(
                    id_val,
                    -32002,
                    &format!("resource not found: {uri}"),
                )),
            }
        }
        _ => Some(err_resp(
            id_val,
            -32601,
            &format!("method not found: {method}"),
        )),
    }
}

fn tool_catalog() -> Vec<Value> {
    vec![
        json!({
            "name": "travel_status",
            "description": "Return AgTravelCraft database status, voyage counts, and Vienna Customs quarantine metrics.",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "travel_seed_demo",
            "description": "Run a full end-to-end Vienna -> Zurich -> Osaka -> Tokyo Craft Council -> Vienna Customs demo voyage (including a flagged injection turn).",
            "inputSchema": { "type": "object", "properties": {} }
        }),
        json!({
            "name": "travel_list_voyages",
            "description": "List recorded diplomatic voyages with their signed transit stamps and council minutes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "minimum": 1, "maximum": 500 }
                }
            }
        }),
        json!({
            "name": "travel_list_customs",
            "description": "List returning council minutes held in Vienna Customs with taint and prompt-injection scan flags.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "minimum": 1, "maximum": 500 }
                }
            }
        }),
        json!({
            "name": "travel_decide_customs",
            "description": "Human-only Vienna Customs gate. Calls over MCP are intentionally blocked (red line #5); humans must approve or reject in the AgTravelCraft GUI or local CLI.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "voyage_id": { "type": "string" },
                    "approve": { "type": "boolean" },
                    "reviewer": { "type": "string" }
                },
                "required": ["voyage_id", "approve", "reviewer"]
            }
        }),
        json!({
            "name": "travel_backup_snapshot",
            "description": "Create an atomic SQLite VACUUM INTO backup snapshot of the AgTravelCraft database.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "destination": { "type": "string" }
                },
                "required": ["destination"]
            }
        }),
        json!({
            "name": "travel_skill_read",
            "description": "Read the bundled AgTravelCraft workflow SKILL.md.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" }
                }
            }
        }),
        json!({
            "name": "travel_build_stage",
            "description": "Build a read-only Resonite (resonite-mcp / ResoniteLink) or Overte (overte-mcp) Diplomatic Teahouse 3D stage script and sanitized ComfyUI postcard prompt for a voyage.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "voyage_id": { "type": "string" },
                    "target": { "type": "string", "enum": ["resonite", "overte"] }
                },
                "required": ["voyage_id", "target"]
            }
        }),
        json!({
            "name": "travel_export_chronicle",
            "description": "Export a Storytold-ready Markdown travelogue chronicle for a voyage, respecting all Vienna Customs turn redactions.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "voyage_id": { "type": "string" }
                },
                "required": ["voyage_id"]
            }
        }),
        json!({
            "name": "travel_redact_customs_turn",
            "description": "Human-only selective turn redaction at Vienna Customs. Calls over MCP are intentionally blocked (red line #5).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "voyage_id": { "type": "string" },
                    "turn_index": { "type": "integer", "minimum": 0 },
                    "reviewer": { "type": "string" }
                },
                "required": ["voyage_id", "turn_index", "reviewer"]
            }
        }),
    ]
}

fn call_tool(store: &TravelStore, name: &str, args: &Value) -> Value {
    let res = match name {
        "travel_status" => Command::Status.dispatch(store),
        "travel_seed_demo" => Command::SeedDemo.dispatch(store),
        "travel_list_voyages" => {
            let limit = args
                .get("limit")
                .and_then(Value::as_u64)
                .and_then(|v| usize::try_from(v).ok());
            Command::ListVoyages { limit }.dispatch(store)
        }
        "travel_list_customs" => {
            let limit = args
                .get("limit")
                .and_then(Value::as_u64)
                .and_then(|v| usize::try_from(v).ok());
            Command::ListCustoms { limit }.dispatch(store)
        }
        "travel_decide_customs" => {
            let voyage_id = args
                .get("voyage_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let approve = args
                .get("approve")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let reviewer = args
                .get("reviewer")
                .and_then(Value::as_str)
                .unwrap_or("mcp-caller")
                .to_string();
            // Red line #5: MCP callers are NEVER human.
            Command::DecideCustoms {
                voyage_id,
                approve,
                reviewer,
                is_human: false,
            }
            .dispatch(store)
        }
        "travel_backup_snapshot" => {
            let destination = args
                .get("destination")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            Command::BackupSnapshot { destination }.dispatch(store)
        }
        "travel_skill_read" => {
            let skill_name = args
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("agtravelcraft-workflow")
                .to_string();
            Command::ReadSkill { name: skill_name }.dispatch(store)
        }
        "travel_build_stage" => {
            let voyage_id = args
                .get("voyage_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let target = args
                .get("target")
                .and_then(Value::as_str)
                .unwrap_or("resonite")
                .to_string();
            Command::BuildStage { voyage_id, target }.dispatch(store)
        }
        "travel_export_chronicle" => {
            let voyage_id = args
                .get("voyage_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            Command::ExportChronicle { voyage_id }.dispatch(store)
        }
        "travel_redact_customs_turn" => {
            let voyage_id = args
                .get("voyage_id")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let turn_index = args
                .get("turn_index")
                .and_then(Value::as_u64)
                .and_then(|v| usize::try_from(v).ok())
                .unwrap_or(0);
            let reviewer = args
                .get("reviewer")
                .and_then(Value::as_str)
                .unwrap_or("mcp-caller")
                .to_string();
            // Red line #5: MCP callers are NEVER human.
            Command::RedactCustomsTurn {
                voyage_id,
                turn_index,
                reviewer,
                is_human: false,
            }
            .dispatch(store)
        }
        _ => {
            return json!({
                "content": [{ "type": "text", "text": format!("unknown tool: {name}") }],
                "isError": true
            });
        }
    };

    match res {
        Ok(val) => {
            let text = serde_json::to_string_pretty(&val).unwrap_or_else(|_| "{}".to_string());
            json!({
                "content": [{ "type": "text", "text": text }],
                "isError": false
            })
        }
        Err(e) => json!({
            "content": [{ "type": "text", "text": e.to_string() }],
            "isError": true
        }),
    }
}

fn ok_resp(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn err_resp(id: Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

#[cfg(test)]
mod tests {
    use agtravelcraft_core::store::TravelStore;
    use serde_json::json;

    use super::{PROTOCOL_VERSION, handle_request};

    #[test]
    fn mcp_initialize_reports_2026_07_28() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let resp = handle_request(
            &store,
            &json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}),
        )
        .ok_or("expected response")?;
        assert_eq!(resp["result"]["protocolVersion"], PROTOCOL_VERSION);
        assert_eq!(resp["result"]["serverInfo"]["name"], "agtravelcraft-mcp");
        Ok(())
    }

    #[test]
    fn mcp_tools_list_contains_all_ten_tools() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let resp = handle_request(
            &store,
            &json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
        )
        .ok_or("expected response")?;
        let tools = resp["result"]["tools"]
            .as_array()
            .ok_or("tools must be array")?;
        assert_eq!(tools.len(), 10);
        Ok(())
    }

    #[test]
    fn mcp_build_stage_and_block_redact() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let voyage = store.seed_demo()?;
        let stage = handle_request(
            &store,
            &json!({
                "jsonrpc":"2.0","id":9,"method":"tools/call",
                "params":{
                    "name":"travel_build_stage",
                    "arguments":{"voyage_id": voyage.voyage_id, "target":"resonite"}
                }
            }),
        )
        .ok_or("expected response")?;
        assert_eq!(stage["result"]["isError"], false);

        let redact = handle_request(
            &store,
            &json!({
                "jsonrpc":"2.0","id":10,"method":"tools/call",
                "params":{
                    "name":"travel_redact_customs_turn",
                    "arguments":{"voyage_id": voyage.voyage_id, "turn_index": 3, "reviewer": "bot"}
                }
            }),
        )
        .ok_or("expected response")?;
        assert_eq!(redact["result"]["isError"], true);
        Ok(())
    }

    #[test]
    fn mcp_resources_list_and_read_work() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let list = handle_request(
            &store,
            &json!({"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}),
        )
        .ok_or("expected response")?;
        let uri = list["result"]["resources"][0]["uri"]
            .as_str()
            .ok_or("missing uri")?
            .to_string();

        let read = handle_request(
            &store,
            &json!({"jsonrpc":"2.0","id":4,"method":"resources/read","params":{"uri": uri}}),
        )
        .ok_or("expected response")?;
        let text = read["result"]["contents"][0]["text"]
            .as_str()
            .ok_or("missing text")?;
        assert!(text.contains("AgTravelCraft"));
        Ok(())
    }

    #[test]
    fn mcp_resources_read_unknown_returns_32002() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let resp = handle_request(
            &store,
            &json!({"jsonrpc":"2.0","id":5,"method":"resources/read","params":{"uri":"skill://agtravelcraft/missing"}}),
        )
        .ok_or("expected response")?;
        assert_eq!(resp["error"]["code"], -32002);
        Ok(())
    }

    #[test]
    fn mcp_seed_demo_and_list_customs() -> Result<(), Box<dyn std::error::Error>> {
        let store = TravelStore::open_in_memory()?;
        let seeded = handle_request(
            &store,
            &json!({
                "jsonrpc":"2.0","id":6,"method":"tools/call",
                "params":{"name":"travel_seed_demo","arguments":{}}
            }),
        )
        .ok_or("expected response")?;
        assert_eq!(seeded["result"]["isError"], false);

        let customs = handle_request(
            &store,
            &json!({
                "jsonrpc":"2.0","id":7,"method":"tools/call",
                "params":{"name":"travel_list_customs","arguments":{}}
            }),
        )
        .ok_or("expected response")?;
        assert_eq!(customs["result"]["isError"], false);
        let text = customs["result"]["content"][0]["text"]
            .as_str()
            .ok_or("missing text")?;
        assert!(text.contains("remote_peer"));
        Ok(())
    }

    #[test]
    fn mcp_decide_customs_is_blocked_with_is_error_true() -> Result<(), Box<dyn std::error::Error>>
    {
        let store = TravelStore::open_in_memory()?;
        let voyage = store.seed_demo()?;
        let resp = handle_request(
            &store,
            &json!({
                "jsonrpc":"2.0","id":8,"method":"tools/call",
                "params":{
                    "name":"travel_decide_customs",
                    "arguments":{
                        "voyage_id": voyage.voyage_id,
                        "approve": true,
                        "reviewer": "rogue-bot"
                    }
                }
            }),
        )
        .ok_or("expected response")?;
        assert_eq!(resp["result"]["isError"], true);
        let msg = resp["result"]["content"][0]["text"]
            .as_str()
            .ok_or("missing error text")?;
        assert!(msg.contains("human-only"));
        Ok(())
    }
}
