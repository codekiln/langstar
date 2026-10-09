//! TEMPORARY diagnostic probe for #748. Prints only HTTP status codes and
//! error bodies (never IDs or names). Remove before merge.
#![cfg(feature = "integration-tests")]

use serde_json::{Value, json};

fn env(k: &str) -> String {
    std::env::var(k).unwrap_or_else(|_| panic!("{k} must be set"))
}

fn redact(body: &str) -> String {
    // Keep error bodies short and drop anything that looks like a UUID.
    let re_like: String = body
        .split('"')
        .map(|s| {
            if s.len() == 36 && s.chars().filter(|c| *c == '-').count() == 4 {
                "<uuid>"
            } else {
                s
            }
        })
        .collect::<Vec<_>>()
        .join("\"");
    re_like.chars().take(400).collect()
}

#[tokio::test]
async fn probe_playground_settings_create() {
    let key = env("LANGSMITH_API_KEY");
    let org = env("LANGSMITH_ORGANIZATION_ID");
    let ws = env("LANGSMITH_WORKSPACE_ID");
    let http = reqwest::Client::new();
    let url = "https://api.smith.langchain.com/api/v1/playground-settings";
    let suffix = uuid::Uuid::new_v4().simple().to_string();

    let lc = json!({
        "lc": 1, "type": "constructor",
        "id": ["langchain", "chat_models", "openai", "ChatOpenAI"],
        "kwargs": {"model": "gpt-4o-mini", "temperature": 0.0}
    });

    let bodies: Vec<(&str, Value)> = vec![
        ("settings-only", json!({"settings": lc})),
        (
            "named",
            json!({"name": format!("probe-{suffix}-a"), "settings": lc}),
        ),
        (
            "named+options",
            json!({"name": format!("probe-{suffix}-b"), "settings": lc,
                   "options": {"requests_per_second": 5}}),
        ),
        (
            "named+type+scope",
            json!({"name": format!("probe-{suffix}-c"), "settings": lc,
                   "settings_type": "complex", "scope": "workspace"}),
        ),
        (
            "simple-type",
            json!({"name": format!("probe-{suffix}-d"), "settings": lc,
                   "settings_type": "simple"}),
        ),
        (
            "kv-settings",
            json!({"name": format!("probe-{suffix}-e"), "settings": {"key": "value"}}),
        ),
    ];

    // header variants: (label, send org, send ws)
    let header_variants = [
        ("org+ws", true, true),
        ("ws-only", false, true),
        ("org-only", true, false),
        ("none", false, false),
    ];

    let mut created = Vec::new();
    for (hlabel, send_org, send_ws) in header_variants {
        for (blabel, body) in &bodies {
            let mut b = body.clone();
            if let Some(n) = b.get("name").and_then(|v| v.as_str()) {
                b["name"] = json!(format!("{n}-{hlabel}"));
            }
            let mut req = http.post(url).header("X-Api-Key", &key).json(&b);
            if send_org {
                req = req.header("x-organization-id", &org);
            }
            if send_ws {
                req = req.header("X-Tenant-Id", &ws);
            }
            let resp = req.send().await.expect("send");
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            if status.is_success() {
                let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                let keys: Vec<_> = v
                    .as_object()
                    .map(|o| o.keys().cloned().collect())
                    .unwrap_or_default();
                println!("PROBE headers={hlabel} body={blabel} -> {status} keys={keys:?}");
                if let Some(id) = v.get("id").and_then(|v| v.as_str()) {
                    created.push((id.to_string(), send_org, send_ws));
                }
            } else {
                println!(
                    "PROBE headers={hlabel} body={blabel} -> {status} {}",
                    redact(&text)
                );
            }
        }
    }

    for (id, send_org, send_ws) in created {
        let mut req = http.delete(format!("{url}/{id}")).header("X-Api-Key", &key);
        if send_org {
            req = req.header("x-organization-id", &org);
        }
        if send_ws {
            req = req.header("X-Tenant-Id", &ws);
        }
        let s = req.send().await.map(|r| r.status());
        println!("PROBE cleanup -> {:?}", s.map(|s| s.as_u16()));
    }

    // Probe GET list with header variants
    for (hlabel, send_org, send_ws) in header_variants {
        let mut req = http.get(url).header("X-Api-Key", &key);
        if send_org {
            req = req.header("x-organization-id", &org);
        }
        if send_ws {
            req = req.header("X-Tenant-Id", &ws);
        }
        let resp = req.send().await.expect("send");
        let status = resp.status();
        let v: Value = resp.json().await.unwrap_or(Value::Null);
        println!(
            "PROBE list headers={hlabel} -> {status} count={}",
            v.as_array().map(|a| a.len()).unwrap_or(0)
        );
    }

    panic!("probe always fails so nextest prints its stdout");
}
