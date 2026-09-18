//! 豆瓣电影/剧集信息查询工具
//!
//! 双击运行后自动拉起本地 HTTP 服务并打开浏览器，
//! 用户在网页中输入豆瓣条目 ID 即可查询电影/剧集信息。
//!
//! 数据来源：豆瓣移动端 API `m.douban.com/rexxar/api/v2/movie/<id>`，
//! 无需登录即可访问（桌面端网页已被安全验证拦截）。

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};
use std::net::SocketAddr;

const HTML: &str = include_str!("../static/index.html");

// 常用浏览器 UA 池：每次请求轮换，避免被识别为固定 UA
const UA_POOL: &[&str] = &[
    "Mozilla/5.0 (Linux; Android 16; Pixel 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/153.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 14; Xiaomi 14) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (Linux; Android 13; HUAWEI Mate 50 Pro) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/145.0.0.0 Mobile Safari/537.36",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 18_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.1 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (iPhone; CPU iPhone OS 17_5 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.5 Mobile/15E148 Safari/604.1",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/151.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/150.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15",
    "Mozilla/5.0 (Linux; Android 14; Pixel 8) Gecko/20240101 Firefox/130.0 Mobile",
    "Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0",
];

use std::sync::atomic::{AtomicUsize, Ordering};
static UA_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// 从 UA 池中轮换选取一个 UA
fn pick_ua() -> &'static str {
    let n = UA_COUNTER.fetch_add(1, Ordering::Relaxed);
    // 叠加启动时间种子，使每次启动后的轮换起点不同
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos() as usize)
        .unwrap_or(0);
    UA_POOL[(n.wrapping_add(seed)) % UA_POOL.len()]
}

fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(pick_ua())
        .default_headers({
            let mut h = reqwest::header::HeaderMap::new();
            h.insert(
                reqwest::header::REFERER,
                reqwest::header::HeaderValue::from_static("https://m.douban.com/"),
            );
            h
        })
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("failed to build http client")
}

/// 从豆瓣移动端 API 抓取条目信息
async fn fetch_subject(client: &reqwest::Client, id: &str) -> Result<Value, String> {
    let api_url = format!("https://m.douban.com/rexxar/api/v2/movie/{}", id);
    let resp = client
        .get(&api_url)
        .send()
        .await
        .map_err(|e| format!("请求豆瓣失败: {}", e))?;

    if resp.status() != StatusCode::OK {
        return Err(format!("豆瓣返回状态码 {}", resp.status()));
    }

    let raw: Value = resp
        .json()
        .await
        .map_err(|e| format!("解析豆瓣响应失败: {}", e))?;

    let resource_type = if raw.get("is_tv").and_then(Value::as_bool).unwrap_or(false) {
        "tv"
    } else {
        "movie"
    };
    let rating_num = raw
        .get("rating")
        .and_then(|r| r.get("value"))
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let title = raw
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let year = raw
        .get("year")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let cover = raw
        .get("pic")
        .and_then(|p| p.get("large"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();

    let names = |key: &str| -> String {
        raw.get(key)
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.get("name").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    };
    let directors = names("directors");

    let mut actors: Vec<String> = raw
        .get("actors")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.get("name").and_then(Value::as_str).map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let mut writers: Vec<String> = Vec::new();

    // 编剧不在主接口，从 credits 演职员表提取（roles 包含"编剧"）
    let credits_url = format!(
        "https://m.douban.com/rexxar/api/v2/movie/{}/credits",
        id
    );
    if let Ok(resp) = client.get(&credits_url).send().await {
        if resp.status() == StatusCode::OK {
            if let Ok(credits) = resp.json::<Value>().await {
                if let Some(items) = credits.get("items").and_then(Value::as_array) {
                    for item in items {
                        let roles = item
                            .get("roles")
                            .and_then(Value::as_array)
                            .map(|a| {
                                a.iter()
                                    .filter_map(|v| v.as_str())
                                    .collect::<Vec<_>>()
                                    .join(",")
                            })
                            .unwrap_or_default();
                        let character = item
                            .get("character")
                            .and_then(Value::as_str)
                            .unwrap_or("");
                        if let Some(name) = item.get("name").and_then(Value::as_str) {
                            if roles.contains("编剧") {
                                writers.push(name.to_string());
                            }
                            // 主接口未返回演员时，从演职员表兜底
                            if actors.is_empty()
                                && (roles.contains("演员") || character.contains("饰"))
                            {
                                actors.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 去重并保持顺序
    let dedup = |v: Vec<String>| -> Vec<String> {
        let mut seen = std::collections::HashSet::new();
        v.into_iter().filter(|x| seen.insert(x.clone())).collect()
    };
    let actors = dedup(actors).join(", ");
    let writers = dedup(writers).join(", ");

    let summary = raw
        .get("intro")
        .and_then(Value::as_str)
        .unwrap_or("")
        .replace(['\n', ' '], "");

    let genres = raw
        .get("genres")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let releases = raw
        .get("pubdate")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let runtime = raw
        .get("durations")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    let translation = raw
        .get("aka")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();

    Ok(json!({
        "id": id,
        "resource_type": resource_type,
        "rating_num": rating_num,
        "title": title,
        "year": year,
        "cover": cover,
        "genres": genres,
        "director": directors,
        "writers": writers,
        "actors": actors,
        "summary": summary,
        "releases": releases,
        "runtime": runtime,
        "translation": translation,
    }))
}

async fn index() -> impl IntoResponse {
    Html(HTML)
}

async fn subject(Path(id): Path<String>) -> impl IntoResponse {
    // 仅接受纯数字 ID，防止 URL 注入
    if !id.chars().all(|c| c.is_ascii_digit()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "无效的豆瓣条目 ID，仅接受数字"})),
        );
    }

    let client = http_client();
    match fetch_subject(&client, &id).await {
        Ok(data) => (StatusCode::OK, Json(data)),
        Err(msg) => (StatusCode::BAD_GATEWAY, Json(json!({"error": msg}))),
    }
}

async fn quit() -> &'static str {
    // 让响应先发出，再退出进程
    std::thread::spawn(|| {
        std::thread::sleep(std::time::Duration::from_millis(200));
        std::process::exit(0);
    });
    "bye"
}

#[tokio::main]
async fn main() {
    // 随机空闲端口，避免与其他程序冲突
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind local port");
    let addr: SocketAddr = listener
        .local_addr()
        .expect("failed to read local addr");
    let url = format!("http://{}/", addr);

    let app = Router::new()
        .route("/", get(index))
        .route("/api/subject/{id}", get(subject))
        .route("/quit", get(quit));

    println!("豆瓣查询工具已启动: {}", url);
    println!("按 Ctrl+C 退出");

    // 稍后自动打开浏览器
    let url_open = url.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(300));
        if let Err(e) = webbrowser::open(&url_open) {
            println!("请手动在浏览器打开: {}", url_open);
            eprintln!("自动打开浏览器失败: {}", e);
        }
    });

    axum::serve(listener, app).await.expect("server error");
}
