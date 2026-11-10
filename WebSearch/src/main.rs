use serde::Deserialize;
use serde_json::{Value, json};
use std::{io::Read, process::Command};

#[derive(Deserialize)]
struct Request {
    api_version: u32,
    action: String,
    #[serde(default)]
    query: String,
    id: Option<String>,
    #[serde(default)]
    config: Value,
    #[serde(default)]
    values: Value,
}

fn search_url(engine: &str, query: &str) -> Result<String, String> {
    let base = match engine {
        "Bing" => "https://www.bing.com/search?q=",
        "Google" => "https://www.google.com/search?q=",
        "百度" => "https://www.baidu.com/s?wd=",
        "DuckDuckGo" => "https://duckduckgo.com/?q=",
        _ => return Err("搜索引擎无效，请在插件设置中重新选择。".into()),
    };
    let query = query.trim();
    if query.is_empty() {
        return Err("请输入搜索内容。".into());
    }
    if query.len() > 4096 {
        return Err("搜索内容超过 4096 字节，请缩短后重试。".into());
    }
    let mut url = String::from(base);
    for byte in query.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            url.push(byte as char);
        } else {
            use std::fmt::Write;
            write!(url, "%{byte:02X}").unwrap();
        }
    }
    Ok(url)
}

fn open_browser(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = Command::new("open");
    #[cfg(target_os = "linux")]
    let mut command = Command::new("xdg-open");
    #[cfg(target_os = "windows")]
    let mut command = {
        use std::os::windows::process::CommandExt;
        let mut command = Command::new("rundll32.exe");
        command.arg("url.dll,FileProtocolHandler");
        command.creation_flags(0x08000000);
        command
    };
    let status = command
        .arg(url)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("无法打开默认浏览器：{e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("无法打开默认浏览器，请检查系统默认浏览器设置。".into())
    }
}

fn suggestions(engine: &str, query: &str) -> (Vec<String>, String) {
    if query.trim().is_empty() {
        return (vec![], "输入关键词以查看搜索推荐".into());
    }
    let encoded = match search_url(engine, query) {
        Ok(url) => url.split_once('=').unwrap().1.to_owned(),
        Err(error) => return (vec![], error),
    };
    let base = match engine {
        "Google" => "https://suggestqueries.google.com/complete/search?client=firefox&q=",
        "百度" => "https://www.baidu.com/sugrec?prod=pc&ie=utf-8&wd=",
        "DuckDuckGo" => "https://duckduckgo.com/ac/?type=list&q=",
        _ => "https://api.bing.com/osjson.aspx?query=",
    };
    let output = Command::new("curl")
        .args([
            "--silent",
            "--fail",
            "--max-time",
            "1.5",
            "--max-filesize",
            "65536",
        ])
        .arg(format!("{base}{encoded}"))
        .output();
    let data = output
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| serde_json::from_slice::<Value>(&o.stdout).ok());
    let mut items = Vec::new();
    if let Some(data) = data {
        let candidates = if engine == "百度" {
            data["g"].as_array()
        } else {
            data[1].as_array()
        };
        for candidate in candidates.into_iter().flatten() {
            let text = if engine == "百度" {
                candidate["q"].as_str()
            } else {
                candidate.as_str()
            };
            if let Some(text) = text.filter(|s| !s.is_empty() && s.len() <= 128) {
                if !items.iter().any(|s| s == text) {
                    items.push(text.to_owned());
                }
            }
            if items.len() == 8 {
                break;
            }
        }
    }
    let message = if items.is_empty() {
        "暂无推荐，可按 Enter 直接搜索"
    } else {
        "点击推荐词搜索"
    };
    (items, message.into())
}

fn panel(query: &str, engine: &str, message: &str, options: Vec<String>) -> Value {
    json!({"title": "WebSearch", "submit": "搜索", "focus": "query", "live": true, "fields": [
        {"key": "query", "label": format!("搜索内容 · {engine} · Enter 搜索"), "type": "line", "value": query},
        {"key": "suggestion", "label": format!("搜索推荐 · {message}"), "type": "suggestions", "value": "", "options": options}
    ]})
}

fn respond(
    request: Request,
    open: impl FnOnce(&str) -> Result<(), String>,
) -> Result<Value, String> {
    if request.api_version != 1 {
        return Err("不支持的插件接口版本".into());
    }
    let engine = request.config["engine"].as_str().unwrap_or("Bing");
    match request.action.as_str() {
        "search" => Ok(
            json!({"results": [{"id": "open", "title": "打开网页搜索栏", "detail": format!("{engine} · 系统默认浏览器 · Enter 打开 WebSearch") }]}),
        ),
        "activate" if request.id.as_deref() == Some("open") => {
            Ok(json!({"panel": panel(&request.query, engine, "输入关键词以查看搜索推荐", vec![])}))
        }
        "panel_change" if request.id.as_deref() == Some("open") => {
            let query = request.values["query"].as_str().unwrap_or("");
            let (options, message) = suggestions(engine, query);
            Ok(json!({"panel": panel(query, engine, &message, options)}))
        }
        "panel_submit" if request.id.as_deref() == Some("open") => {
            let query = request.values["suggestion"]
                .as_str()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| request.values["query"].as_str().unwrap_or(""));
            let message = match search_url(engine, query).and_then(|url| open(&url)) {
                Ok(()) => "已在默认浏览器中打开搜索结果。".into(),
                Err(error) => error,
            };
            Ok(json!({"panel": panel(query, engine, &message, vec![])}))
        }
        _ => Err("未知插件请求".into()),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    std::io::stdin()
        .take(64 * 1024 + 1)
        .read_to_string(&mut input)?;
    if input.len() > 64 * 1024 {
        return Err("插件请求过大".into());
    }
    let response = respond(serde_json::from_str(&input)?, open_browser)?;
    println!("{response}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommendation_click_searches_selected_text_and_empty_input_stays_offline() {
        let request = serde_json::from_value(json!({"api_version":1,"action":"panel_submit","id":"open","config":{"engine":"Bing"},"values":{"query":"original","suggestion":"recommended"}})).unwrap();
        let result = respond(request, |url| {
            assert_eq!(url, "https://www.bing.com/search?q=recommended");
            Ok(())
        })
        .unwrap();
        assert_eq!(result["panel"]["fields"][0]["value"], "recommended");
        let request = serde_json::from_value(
            json!({"api_version":1,"action":"panel_change","id":"open","values":{"query":""}}),
        )
        .unwrap();
        let result = respond(request, |_| panic!("suggestions must not launch browser")).unwrap();
        assert_eq!(result["panel"]["fields"][1]["options"], json!([]));
        assert_eq!(result["panel"]["fields"][1]["type"], "suggestions");
    }

    #[test]
    fn encodes_unicode_and_url_delimiters() {
        assert_eq!(
            search_url("Google", " 中文 &+#?/ ").unwrap(),
            "https://www.google.com/search?q=%E4%B8%AD%E6%96%87%20%26%2B%23%3F%2F"
        );
        for engine in ["Bing", "百度", "DuckDuckGo"] {
            assert!(search_url(engine, "rust").unwrap().ends_with("rust"));
        }
        assert!(search_url("unknown", "rust").is_err());
        assert!(search_url("Bing", "  ").is_err());
        assert!(search_url("Bing", &"a".repeat(4097)).is_err());
    }

    #[test]
    fn protocol_opens_panel_and_submits_with_saved_engine() {
        let request = |action: &str| {
            serde_json::from_value(json!({"api_version":1,"action":action,"id":"open","query":"预填","config":{"engine":"百度"},"values":{"query":"hello & world"}})).unwrap()
        };
        let result = respond(request("activate"), |_| {
            panic!("activation must only open panel")
        })
        .unwrap();
        assert_eq!(result["panel"]["fields"][0]["value"], "预填");
        assert_eq!(result["panel"]["focus"], "query");
        let result = respond(request("panel_submit"), |url| {
            assert_eq!(url, "https://www.baidu.com/s?wd=hello%20%26%20world");
            Err("浏览器启动失败".into())
        })
        .unwrap();
        assert_eq!(result["panel"]["fields"][0]["value"], "hello & world");
        assert!(
            result["panel"]["fields"][1]["label"]
                .as_str()
                .unwrap()
                .contains("浏览器启动失败")
        );
    }
}
