use std::time::Duration;

use reqwest::{header::LOCATION, redirect::Policy, Client};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::State;

use crate::settings::{schema::ContentConnectorProfile, SettingsState};

use super::{credential, web_url};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const MAX_RESULTS: u32 = 20;
const MAX_FEED_BYTES: usize = 5 * 1024 * 1024;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverConnectorDescriptor {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub configured: bool,
    pub requires_key: bool,
    pub query_hint: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverResult {
    pub connector_id: String,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub snippet: String,
    pub content_text: String,
}

fn descriptor(id: &str, label: &str, kind: &str, requires_key: bool, query_hint: &str) -> DiscoverConnectorDescriptor {
    let configured = if requires_key {
        credential::load(id).map(|value| !value.trim().is_empty()).unwrap_or(false)
    } else {
        true
    };
    DiscoverConnectorDescriptor {
        id: id.into(),
        label: label.into(),
        kind: kind.into(),
        configured,
        requires_key,
        query_hint: query_hint.into(),
    }
}

#[tauri::command]
pub fn discover_connectors(state: State<'_, SettingsState>) -> Result<Vec<DiscoverConnectorDescriptor>, String> {
    let config = state.snapshot()?;
    let mut items = vec![
        descriptor("wikipedia", "Wikipedia", "public", false, "搜索百科主题"),
        descriptor("arxiv", "arXiv", "public", false, "搜索论文主题"),
        descriptor("stackexchange", "Stack Overflow", "public", false, "搜索技术问题"),
        descriptor("hacker-news", "Hacker News", "public", false, "搜索技术文章"),
        descriptor("rss", "RSS / Atom", "feed", false, "粘贴 RSS / Atom 地址"),
        descriptor("brave", "Brave Search", "paid", true, "搜索互联网"),
        descriptor("tavily", "Tavily", "paid", true, "搜索互联网"),
        descriptor("exa", "Exa", "paid", true, "搜索互联网"),
    ];
    items.extend(config.connectors.into_iter().filter(|item| item.enabled).map(|item| DiscoverConnectorDescriptor {
        id: item.id,
        label: item.name,
        kind: "custom".into(),
        configured: true,
        requires_key: item.api_key_header.as_deref().is_some_and(|value| !value.trim().is_empty()),
        query_hint: "输入查询".into(),
    }));
    Ok(items)
}

#[tauri::command]
pub async fn discover_search(
    state: State<'_, SettingsState>,
    connector_id: String,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<DiscoverResult>, String> {
    let connector_id = connector_id.trim().to_ascii_lowercase();
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err("search query is empty".into());
    }
    let limit = limit.unwrap_or(10).clamp(1, MAX_RESULTS);
    let config = state.snapshot()?;
    let client = Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent("LearnWeave/1.5 knowledge discovery")
        .build()
        .map_err(|error| format!("failed to create connector client: {error}"))?;

    match connector_id.as_str() {
        "wikipedia" => wikipedia(&client, &query, limit).await,
        "arxiv" => arxiv(&client, &query, limit).await,
        "stackexchange" => stackexchange(&client, &query, limit).await,
        "hacker-news" => hacker_news(&client, &query, limit).await,
        "rss" => rss(&query, limit).await,
        "brave" => brave(&client, &query, limit).await,
        "tavily" => tavily(&client, &query, limit).await,
        "exa" => exa(&client, &query, limit).await,
        _ => {
            let profile = config.connectors.iter()
                .find(|item| item.enabled && item.id.eq_ignore_ascii_case(&connector_id))
                .ok_or_else(|| format!("unknown or disabled connector '{connector_id}'"))?;
            custom_json(&client, profile, &query, limit).await
        }
    }
}

async fn json_response(response: reqwest::Response, label: &str) -> Result<Value, String> {
    let status = response.status();
    if !status.is_success() {
        return Err(format!("{label} returned HTTP {}", status.as_u16()));
    }
    response.json::<Value>().await.map_err(|error| format!("failed to parse {label} response: {error}"))
}

fn value_text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.trim().to_string(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(value)) => value.to_string(),
        _ => String::new(),
    }
}

fn strip_markup(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut in_tag = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => { in_tag = false; output.push(' '); }
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    decode_entities(&output).split_whitespace().collect::<Vec<_>>().join(" ")
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

fn clipped(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars { value.to_string() } else { value.chars().take(max_chars).collect() }
}

async fn wikipedia(client: &Client, query: &str, limit: u32) -> Result<Vec<DiscoverResult>, String> {
    let limit_string = limit.to_string();
    let response = client.get("https://en.wikipedia.org/w/api.php").query(&[
        ("action", "query"),
        ("generator", "search"),
        ("gsrsearch", query),
        ("gsrlimit", limit_string.as_str()),
        ("prop", "extracts"),
        ("exintro", "1"),
        ("explaintext", "1"),
        ("format", "json"),
    ]).send().await.map_err(|error| format!("Wikipedia request failed: {error}"))?;
    let value = json_response(response, "Wikipedia").await?;
    let pages = value.pointer("/query/pages").and_then(Value::as_object);
    Ok(pages.into_iter().flat_map(|pages| pages.values()).filter_map(|page| {
        let title = value_text(page.get("title"));
        if title.is_empty() { return None; }
        let text = value_text(page.get("extract"));
        let mut url = reqwest::Url::parse("https://en.wikipedia.org/wiki/").ok()?;
        url.path_segments_mut().ok()?.push(&title.replace(' ', "_"));
        Some(DiscoverResult {
            connector_id: "wikipedia".into(),
            title,
            url: Some(url.to_string()),
            author: Some("Wikipedia contributors".into()),
            snippet: clipped(&text, 500),
            content_text: clipped(&text, 12_000),
        })
    }).take(limit as usize).collect())
}

async fn stackexchange(client: &Client, query: &str, limit: u32) -> Result<Vec<DiscoverResult>, String> {
    let response = client.get("https://api.stackexchange.com/2.3/search/advanced").query(&[
        ("site", "stackoverflow"),
        ("order", "desc"),
        ("sort", "relevance"),
        ("q", query),
        ("filter", "withbody"),
        ("pagesize", &limit.to_string()),
    ]).send().await.map_err(|error| format!("Stack Exchange request failed: {error}"))?;
    let value = json_response(response, "Stack Exchange").await?;
    Ok(value.get("items").and_then(Value::as_array).into_iter().flatten().filter_map(|item| {
        let title = strip_markup(&value_text(item.get("title")));
        if title.is_empty() { return None; }
        let body = strip_markup(&value_text(item.get("body")));
        Some(DiscoverResult {
            connector_id: "stackexchange".into(),
            title,
            url: nonempty(value_text(item.get("link"))),
            author: item.pointer("/owner/display_name").and_then(Value::as_str).map(strip_markup),
            snippet: clipped(&body, 500),
            content_text: clipped(&body, 12_000),
        })
    }).take(limit as usize).collect())
}

async fn hacker_news(client: &Client, query: &str, limit: u32) -> Result<Vec<DiscoverResult>, String> {
    let response = client.get("https://hn.algolia.com/api/v1/search").query(&[
        ("query", query),
        ("tags", "story"),
        ("hitsPerPage", &limit.to_string()),
    ]).send().await.map_err(|error| format!("Hacker News request failed: {error}"))?;
    let value = json_response(response, "Hacker News").await?;
    Ok(value.get("hits").and_then(Value::as_array).into_iter().flatten().filter_map(|item| {
        let title = value_text(item.get("title"));
        if title.is_empty() { return None; }
        let object_id = value_text(item.get("objectID"));
        let url = nonempty(value_text(item.get("url"))).or_else(|| (!object_id.is_empty()).then(|| format!("https://news.ycombinator.com/item?id={object_id}")));
        let story = strip_markup(&value_text(item.get("story_text")));
        Some(DiscoverResult {
            connector_id: "hacker-news".into(),
            title,
            url,
            author: nonempty(value_text(item.get("author"))),
            snippet: clipped(&story, 500),
            content_text: if story.is_empty() { "Hacker News story link".into() } else { clipped(&story, 12_000) },
        })
    }).take(limit as usize).collect())
}

async fn arxiv(client: &Client, query: &str, limit: u32) -> Result<Vec<DiscoverResult>, String> {
    let response = client.get("https://export.arxiv.org/api/query").query(&[
        ("search_query", format!("all:{query}")),
        ("start", "0".to_string()),
        ("max_results", limit.to_string()),
        ("sortBy", "relevance".to_string()),
    ]).send().await.map_err(|error| format!("arXiv request failed: {error}"))?;
    if !response.status().is_success() {
        return Err(format!("arXiv returned HTTP {}", response.status().as_u16()));
    }
    let body = response.text().await.map_err(|error| format!("failed to read arXiv response: {error}"))?;
    Ok(xml_blocks(&body, "entry").into_iter().filter_map(|entry| {
        let title = xml_value(entry, "title");
        if title.is_empty() { return None; }
        let summary = xml_value(entry, "summary");
        let author = xml_blocks(entry, "author").first().map(|block| xml_value(block, "name")).filter(|value| !value.is_empty());
        Some(DiscoverResult {
            connector_id: "arxiv".into(),
            title,
            url: nonempty(xml_value(entry, "id")),
            author,
            snippet: clipped(&summary, 500),
            content_text: clipped(&summary, 12_000),
        })
    }).take(limit as usize).collect())
}

async fn rss(url: &str, limit: u32) -> Result<Vec<DiscoverResult>, String> {
    let body = fetch_public_feed(url).await?;
    let blocks = {
        let items = xml_blocks(&body, "item");
        if items.is_empty() { xml_blocks(&body, "entry") } else { items }
    };
    Ok(blocks.into_iter().filter_map(|entry| {
        let title = xml_value(entry, "title");
        if title.is_empty() { return None; }
        let link = {
            let direct = xml_value(entry, "link");
            if !direct.is_empty() { direct } else { xml_attr(entry, "link", "href") }
        };
        let content = ["description", "summary", "content"].into_iter()
            .map(|tag| xml_value(entry, tag))
            .find(|value| !value.is_empty()).unwrap_or_default();
        let author = ["author", "dc:creator"].into_iter().map(|tag| xml_value(entry, tag)).find(|value| !value.is_empty());
        Some(DiscoverResult {
            connector_id: "rss".into(),
            title,
            url: nonempty(link),
            author,
            snippet: clipped(&strip_markup(&content), 500),
            content_text: clipped(&strip_markup(&content), 12_000),
        })
    }).take(limit as usize).collect())
}

async fn brave(client: &Client, query: &str, limit: u32) -> Result<Vec<DiscoverResult>, String> {
    let key = require_key("brave")?;
    let response = client.get("https://api.search.brave.com/res/v1/web/search")
        .header("X-Subscription-Token", key)
        .query(&[("q", query), ("count", &limit.to_string()), ("extra_snippets", "true")])
        .send().await.map_err(|error| format!("Brave Search request failed: {error}"))?;
    let value = json_response(response, "Brave Search").await?;
    Ok(value.pointer("/web/results").and_then(Value::as_array).into_iter().flatten().filter_map(|item| {
        let title = strip_markup(&value_text(item.get("title")));
        if title.is_empty() { return None; }
        let snippet = strip_markup(&value_text(item.get("description")));
        Some(DiscoverResult {
            connector_id: "brave".into(), title,
            url: nonempty(value_text(item.get("url"))), author: None,
            snippet: clipped(&snippet, 500), content_text: clipped(&snippet, 12_000)
        })
    }).take(limit as usize).collect())
}

async fn tavily(client: &Client, query: &str, limit: u32) -> Result<Vec<DiscoverResult>, String> {
    let key = require_key("tavily")?;
    let response = client.post("https://api.tavily.com/search")
        .json(&json!({"api_key": key, "query": query, "max_results": limit, "search_depth": "basic", "include_raw_content": false}))
        .send().await.map_err(|error| format!("Tavily request failed: {error}"))?;
    let value = json_response(response, "Tavily").await?;
    Ok(value.get("results").and_then(Value::as_array).into_iter().flatten().filter_map(|item| {
        let title = value_text(item.get("title")); if title.is_empty() { return None; }
        let content = value_text(item.get("content"));
        Some(DiscoverResult { connector_id:"tavily".into(), title, url: nonempty(value_text(item.get("url"))), author:None, snippet:clipped(&content,500), content_text:clipped(&content,12_000) })
    }).take(limit as usize).collect())
}

async fn exa(client: &Client, query: &str, limit: u32) -> Result<Vec<DiscoverResult>, String> {
    let key = require_key("exa")?;
    let response = client.post("https://api.exa.ai/search")
        .header("x-api-key", key)
        .json(&json!({"query": query, "numResults": limit, "contents": {"text": {"maxCharacters": 12000}}}))
        .send().await.map_err(|error| format!("Exa request failed: {error}"))?;
    let value = json_response(response, "Exa").await?;
    Ok(value.get("results").and_then(Value::as_array).into_iter().flatten().filter_map(|item| {
        let title = value_text(item.get("title")); if title.is_empty() { return None; }
        let text = value_text(item.get("text"));
        Some(DiscoverResult { connector_id:"exa".into(), title, url: nonempty(value_text(item.get("url"))), author: nonempty(value_text(item.get("author"))), snippet:clipped(&text,500), content_text:clipped(&text,12_000) })
    }).take(limit as usize).collect())
}

async fn custom_json(client: &Client, profile: &ContentConnectorProfile, query: &str, limit: u32) -> Result<Vec<DiscoverResult>, String> {
    let mut request = client.get(profile.endpoint.trim()).query(&[(profile.query_param.as_str(), query)]);
    if let Some(header) = profile.api_key_header.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        let key = credential::load(&profile.id)?;
        if key.trim().is_empty() { return Err(format!("Connector '{}' needs an API key", profile.name)); }
        request = request.header(header, key);
    }
    let value = json_response(request.send().await.map_err(|error| format!("custom connector request failed: {error}"))?, &profile.name).await?;
    let items = json_path(&value, &profile.items_path).and_then(Value::as_array).ok_or_else(|| format!("Connector '{}' itemsPath did not resolve to an array", profile.name))?;
    Ok(items.iter().filter_map(|item| {
        let title = value_text(json_path(item, &profile.title_path)); if title.is_empty() { return None; }
        let snippet = value_text(json_path(item, &profile.snippet_path));
        let content = value_text(json_path(item, &profile.content_path));
        Some(DiscoverResult {
            connector_id: profile.id.clone(), title,
            url: nonempty(value_text(json_path(item, &profile.url_path))),
            author: None,
            snippet: clipped(if snippet.is_empty() { &content } else { &snippet }, 500),
            content_text: clipped(if content.is_empty() { &snippet } else { &content }, 12_000),
        })
    }).take(limit as usize).collect())
}

fn require_key(id: &str) -> Result<String, String> {
    let key = credential::load(id)?;
    if key.trim().is_empty() { Err(format!("{id} API key is not configured")) } else { Ok(key) }
}

fn nonempty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn json_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    for part in path.trim().trim_matches('.').split('.').filter(|part| !part.is_empty()) {
        current = current.get(part)?;
    }
    Some(current)
}

fn xml_blocks<'a>(source: &'a str, tag: &str) -> Vec<&'a str> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let mut rest = source;
    let mut blocks = Vec::new();
    while let Some(start) = rest.find(&open) {
        let after_open = &rest[start..];
        let Some(open_end) = after_open.find('>') else { break; };
        let content = &after_open[open_end + 1..];
        let Some(end) = content.find(&close) else { break; };
        blocks.push(&content[..end]);
        rest = &content[end + close.len()..];
    }
    blocks
}

fn xml_value(source: &str, tag: &str) -> String {
    xml_blocks(source, tag).first().map(|value| {
        decode_entities(value.trim().trim_start_matches("<![CDATA[").trim_end_matches("]]>")).split_whitespace().collect::<Vec<_>>().join(" ")
    }).unwrap_or_default()
}

fn xml_attr(source: &str, tag: &str, attr: &str) -> String {
    let marker = format!("<{tag}");
    let Some(start) = source.find(&marker) else { return String::new(); };
    let rest = &source[start..];
    let Some(end) = rest.find('>') else { return String::new(); };
    let head = &rest[..end];
    for quote in ['"', '\''] {
        let needle = format!("{attr}={quote}");
        if let Some(pos) = head.find(&needle) {
            let value = &head[pos + needle.len()..];
            if let Some(stop) = value.find(quote) { return decode_entities(&value[..stop]); }
        }
    }
    String::new()
}

async fn fetch_public_feed(input: &str) -> Result<String, String> {
    let mut current = web_url::parse_public_url(input)?;
    for redirect_count in 0..=5 {
        let (host, resolved) = web_url::validate_public_target(&current).await?;
        let mut builder = Client::builder().redirect(Policy::none()).timeout(REQUEST_TIMEOUT).user_agent("LearnWeave/1.5 RSS");
        if let Some(address) = resolved { builder = builder.resolve(&host, address); }
        let response = builder.build().map_err(|error| format!("failed to create RSS client: {error}"))?
            .get(current.clone()).send().await.map_err(|error| format!("failed to fetch RSS feed: {error}"))?;
        if response.status().is_redirection() {
            if redirect_count == 5 { return Err("RSS feed redirected too many times".into()); }
            let location = response.headers().get(LOCATION).ok_or_else(|| "RSS redirect had no Location header".to_string())?
                .to_str().map_err(|_| "RSS redirect Location is invalid".to_string())?;
            current = current.join(location).map_err(|error| format!("RSS redirect URL is invalid: {error}"))?;
            continue;
        }
        if !response.status().is_success() { return Err(format!("RSS feed returned HTTP {}", response.status().as_u16())); }
        if response.content_length().is_some_and(|size| size > MAX_FEED_BYTES as u64) { return Err("RSS feed exceeds 5 MB".into()); }
        let bytes = response.bytes().await.map_err(|error| format!("failed to read RSS feed: {error}"))?;
        if bytes.len() > MAX_FEED_BYTES { return Err("RSS feed exceeds 5 MB".into()); }
        return Ok(String::from_utf8_lossy(&bytes).to_string());
    }
    Err("RSS fetch ended unexpectedly".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_path_reads_nested_values() {
        let value = json!({"data":{"items":[{"title":"one"}]}});
        assert_eq!(json_path(&value, "data.items").and_then(Value::as_array).unwrap().len(), 1);
    }

    #[test]
    fn atom_entries_are_parsed_without_xml_dependency() {
        let xml = "<feed><entry><title>A paper</title><summary>Useful abstract</summary><id>https://arxiv.org/abs/1</id><author><name>Alice</name></author></entry></feed>";
        let entry = xml_blocks(xml, "entry")[0];
        assert_eq!(xml_value(entry, "title"), "A paper");
        assert_eq!(xml_value(entry, "summary"), "Useful abstract");
    }
}
