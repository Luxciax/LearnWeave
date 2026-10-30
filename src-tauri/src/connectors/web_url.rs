use std::{net::IpAddr, time::Duration};

use futures_util::StreamExt;
use reqwest::{header::LOCATION, redirect::Policy, Client, Url};
use serde::Serialize;
use tokio::{net::lookup_host, time::timeout};

use crate::knowledge::SourceCaptureInput;

use super::{Capabilities, ConnectorError, FetchedContent, SourceConnector};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);
const DNS_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_REDIRECTS: usize = 5;
const MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;
const MAX_CAPTURE_CHARS: usize = 200_000;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebUrlFetchResult {
    pub url: String,
    pub title: Option<String>,
    pub text: String,
}

pub struct WebUrlConnector;

impl SourceConnector for WebUrlConnector {
    fn id(&self) -> &'static str {
        "web-url"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities { search: false, fetch: true }
    }

    async fn fetch(&self, locator: &str) -> Result<FetchedContent, ConnectorError> {
        let fetched = web_url_fetch(locator.to_string()).await.map_err(ConnectorError::from)?;
        Ok(FetchedContent {
            url: Some(fetched.url),
            title: fetched.title,
            author: None,
            text: fetched.text,
        })
    }

    fn to_source(&self, content: &FetchedContent, selection: Option<&str>) -> SourceCaptureInput {
        SourceCaptureInput {
            platform: "web".into(),
            source_kind: Some("web".into()),
            connector_id: Some(self.id().into()),
            selected_text: selection.unwrap_or(&content.text).to_string(),
            url: content.url.clone(),
            title: content.title.clone(),
            author: content.author.clone(),
            context_before: None,
            context_after: None,
            application: None,
            window_title: None,
        }
    }
}

#[tauri::command]
pub async fn web_url_fetch(url: String) -> Result<WebUrlFetchResult, String> {
    let mut current = parse_public_url(&url)?;

    for redirect_count in 0..=MAX_REDIRECTS {
        let (host, resolved) = validate_public_target(&current).await?;
        let mut client_builder = Client::builder()
            .redirect(Policy::none())
            .timeout(REQUEST_TIMEOUT)
            .user_agent("LearnWeave/1.5 local knowledge capture");
        if let Some(address) = resolved {
            client_builder = client_builder.resolve(&host, address);
        }
        let client = client_builder
            .build()
            .map_err(|error| format!("failed to create web client: {error}"))?;
        let response = client
            .get(current.clone())
            .send()
            .await
            .map_err(|error| format!("failed to fetch URL: {error}"))?;

        if response.status().is_redirection() {
            if redirect_count == MAX_REDIRECTS {
                return Err("URL redirected too many times".into());
            }
            let location = response
                .headers()
                .get(LOCATION)
                .ok_or_else(|| "redirect response did not include a Location header".to_string())?
                .to_str()
                .map_err(|_| "redirect Location header is invalid".to_string())?;
            current = current
                .join(location)
                .map_err(|error| format!("redirect URL is invalid: {error}"))?;
            if !matches!(current.scheme(), "http" | "https") {
                return Err("redirected URL must use http or https".into());
            }
            continue;
        }

        if !response.status().is_success() {
            return Err(format!("URL returned HTTP {}", response.status().as_u16()));
        }

        if response
            .content_length()
            .is_some_and(|length| length > MAX_RESPONSE_BYTES as u64)
        {
            return Err("web page is larger than the 5 MB capture limit".into());
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("text/html")
            .to_ascii_lowercase();
        if !(content_type.starts_with("text/html")
            || content_type.starts_with("text/plain")
            || content_type.starts_with("application/xhtml+xml"))
        {
            return Err("URL does not point to a supported text or HTML page".into());
        }

        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| format!("failed while reading web page: {error}"))?;
            if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES {
                return Err("web page is larger than the 5 MB capture limit".into());
            }
            bytes.extend_from_slice(&chunk);
        }

        let body = String::from_utf8_lossy(&bytes);
        let (title, text) = if content_type.starts_with("text/plain") {
            (None, truncate_chars(body.trim(), MAX_CAPTURE_CHARS))
        } else {
            extract_html(&body)
        };
        if text.trim().is_empty() {
            return Err("could not extract readable text from this page".into());
        }

        return Ok(WebUrlFetchResult {
            url: current.to_string(),
            title,
            text,
        });
    }

    Err("URL fetch ended unexpectedly".into())
}

pub(crate) fn parse_public_url(input: &str) -> Result<Url, String> {
    let url = Url::parse(input.trim()).map_err(|error| format!("invalid URL: {error}"))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err("URL must use http or https".into());
    }
    if url.host_str().is_none() {
        return Err("URL must include a host".into());
    }
    Ok(url)
}

pub(crate) async fn validate_public_target(
    url: &Url,
) -> Result<(String, Option<std::net::SocketAddr>), String> {
    let host = url
        .host_str()
        .ok_or_else(|| "URL must include a host".to_string())?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if host == "localhost" || host.ends_with(".localhost") {
        return Err("local addresses cannot be captured by URL".into());
    }

    if let Ok(ip) = host.parse::<IpAddr>() {
        if !is_public_ip(ip) {
            return Err("private or local network addresses cannot be captured by URL".into());
        }
        return Ok((host, None));
    }

    let port = url
        .port_or_known_default()
        .ok_or_else(|| "URL uses an unsupported port".to_string())?;
    let addresses = timeout(DNS_TIMEOUT, lookup_host((host.as_str(), port)))
        .await
        .map_err(|_| "DNS lookup timed out".to_string())?
        .map_err(|error| format!("failed to resolve URL host: {error}"))?
        .collect::<Vec<_>>();
    if addresses.is_empty() {
        return Err("URL host did not resolve to an address".into());
    }
    if addresses.iter().any(|address| !is_public_ip(address.ip())) {
        return Err("URL host resolves to a private or local network address".into());
    }
    Ok((host, addresses.first().copied()))
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(address) => {
            let octets = address.octets();
            !(octets[0] == 0
                || octets[0] == 10
                || octets[0] == 127
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 169 && octets[1] == 254)
                || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
                || (octets[0] == 192 && octets[1] == 168)
                || (octets[0] == 198 && (octets[1] == 18 || octets[1] == 19))
                || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
                || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
                || octets[0] >= 224)
        }
        IpAddr::V6(address) => {
            if let Some(v4) = address.to_ipv4_mapped() {
                return is_public_ip(IpAddr::V4(v4));
            }
            let octets = address.octets();
            !(address.is_unspecified()
                || address.is_loopback()
                || address.is_multicast()
                || (octets[0] & 0xfe) == 0xfc
                || (octets[0] == 0xfe && (octets[1] & 0xc0) == 0x80)
                || (octets[0] == 0x20
                    && octets[1] == 0x01
                    && octets[2] == 0x0d
                    && octets[3] == 0xb8))
        }
    }
}

fn extract_html(body: &str) -> (Option<String>, String) {
    let mut text = String::with_capacity(body.len().min(MAX_CAPTURE_CHARS * 2));
    let mut title = String::new();
    let mut tag = String::new();
    let mut in_tag = false;
    let mut in_title = false;
    let mut ignored_depth = 0usize;

    for character in body.chars() {
        if in_tag {
            if character == '>' {
                process_tag(
                    &tag,
                    &mut text,
                    &mut in_title,
                    &mut ignored_depth,
                );
                tag.clear();
                in_tag = false;
            } else if tag.len() < 512 {
                tag.push(character);
            }
            continue;
        }
        if character == '<' {
            in_tag = true;
            tag.clear();
            continue;
        }
        if ignored_depth > 0 {
            continue;
        }
        if in_title {
            title.push(character);
        } else {
            text.push(character);
        }
    }

    let title = normalize_inline_text(&decode_html_entities(&title));
    let text = normalize_block_text(&decode_html_entities(&text));
    (
        (!title.is_empty()).then_some(title),
        truncate_chars(&text, MAX_CAPTURE_CHARS),
    )
}

fn process_tag(
    raw: &str,
    text: &mut String,
    in_title: &mut bool,
    ignored_depth: &mut usize,
) {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.starts_with('!') || trimmed.starts_with('?') {
        return;
    }
    let closing = trimmed.starts_with('/');
    let self_closing = trimmed.ends_with('/');
    let name = trimmed
        .trim_start_matches('/')
        .split_ascii_whitespace()
        .next()
        .unwrap_or("")
        .trim_end_matches('/')
        .to_ascii_lowercase();
    if name.is_empty() {
        return;
    }

    let ignored = matches!(
        name.as_str(),
        "script" | "style" | "noscript" | "svg" | "template"
    );
    if closing {
        if ignored && *ignored_depth > 0 {
            *ignored_depth -= 1;
        }
        if name == "title" {
            *in_title = false;
        }
    } else {
        if name == "title" && *ignored_depth == 0 {
            *in_title = true;
        }
        if ignored && !self_closing {
            *ignored_depth += 1;
        }
    }

    if *ignored_depth == 0
        && matches!(
            name.as_str(),
            "p" | "div" | "br" | "li" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                | "article" | "section" | "main" | "blockquote" | "pre" | "tr"
        )
    {
        text.push('\n');
    }
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&#160;", " ")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn normalize_inline_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_block_text(value: &str) -> String {
    let mut blocks = Vec::new();
    for line in value.lines() {
        let line = normalize_inline_text(line);
        if line.is_empty() || blocks.last().is_some_and(|last| last == &line) {
            continue;
        }
        blocks.push(line);
    }
    blocks.join("\n\n")
}

fn truncate_chars(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }
    value.chars().take(limit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_private_ipv4_ranges() {
        for value in [
            "127.0.0.1",
            "10.0.0.1",
            "172.16.2.3",
            "192.168.1.1",
            "169.254.1.2",
            "100.64.0.1",
        ] {
            assert!(!is_public_ip(value.parse().unwrap()), "{value}");
        }
        assert!(is_public_ip("8.8.8.8".parse().unwrap()));
    }

    #[test]
    fn extracts_readable_html_blocks() {
        let html = "<html><head><title> Demo </title><script>bad()</script></head><body><main><h1>Hello</h1><p>First <b>paragraph</b>.</p><p>第二段</p></main></body></html>";
        let (title, text) = extract_html(html);
        assert_eq!(title.as_deref(), Some("Demo"));
        assert!(text.contains("Hello"));
        assert!(text.contains("First paragraph."));
        assert!(text.contains("第二段"));
        assert!(!text.contains("bad()"));
    }
}
