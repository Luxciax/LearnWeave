use std::fmt;

use serde::Serialize;

use crate::knowledge::SourceCaptureInput;

pub mod credential;
pub mod discovery;
pub mod web_url;
pub mod zhihu;

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub search: bool,
    pub fetch: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ContentRef {
    pub locator: String,
    pub title: String,
    pub snippet: String,
    pub content_text: String,
    pub author: Option<String>,
    pub url: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FetchedContent {
    pub url: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectorError {
    Unsupported,
    Message(String),
}

impl fmt::Display for ConnectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsupported => f.write_str("connector capability is unsupported"),
            Self::Message(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for ConnectorError {}

impl From<String> for ConnectorError {
    fn from(value: String) -> Self {
        Self::Message(value)
    }
}

#[allow(async_fn_in_trait)]
pub trait SourceConnector: Send + Sync {
    fn id(&self) -> &str;
    fn capabilities(&self) -> Capabilities;

    async fn search(&self, _query: &str, _limit: u32) -> Result<Vec<ContentRef>, ConnectorError> {
        Err(ConnectorError::Unsupported)
    }

    async fn fetch(&self, _locator: &str) -> Result<FetchedContent, ConnectorError> {
        Err(ConnectorError::Unsupported)
    }

    fn to_source(&self, content: &FetchedContent, selection: Option<&str>) -> SourceCaptureInput;
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectorDescriptor {
    pub id: &'static str,
    pub capabilities: Capabilities,
}

pub const BUILTIN_CONNECTORS: &[ConnectorDescriptor] = &[
    ConnectorDescriptor { id: "web-url", capabilities: Capabilities { search: false, fetch: true } },
    ConnectorDescriptor { id: "zhihu", capabilities: Capabilities { search: true, fetch: true } },
    ConnectorDescriptor { id: "wikipedia", capabilities: Capabilities { search: true, fetch: false } },
    ConnectorDescriptor { id: "arxiv", capabilities: Capabilities { search: true, fetch: false } },
    ConnectorDescriptor { id: "stackexchange", capabilities: Capabilities { search: true, fetch: false } },
    ConnectorDescriptor { id: "hacker-news", capabilities: Capabilities { search: true, fetch: false } },
    ConnectorDescriptor { id: "rss", capabilities: Capabilities { search: true, fetch: false } },
    ConnectorDescriptor { id: "brave", capabilities: Capabilities { search: true, fetch: false } },
    ConnectorDescriptor { id: "tavily", capabilities: Capabilities { search: true, fetch: false } },
    ConnectorDescriptor { id: "exa", capabilities: Capabilities { search: true, fetch: false } },
];
