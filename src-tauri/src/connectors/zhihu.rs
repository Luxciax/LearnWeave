use crate::{knowledge::SourceCaptureInput, zhihu_search};

use super::{Capabilities, ConnectorError, ContentRef, FetchedContent, SourceConnector};

pub struct ZhihuConnector;

impl SourceConnector for ZhihuConnector {
    fn id(&self) -> &'static str {
        "zhihu"
    }

    fn capabilities(&self) -> Capabilities {
        Capabilities { search: true, fetch: true }
    }

    async fn search(&self, query: &str, limit: u32) -> Result<Vec<ContentRef>, ConnectorError> {
        let result = zhihu_search::zhihu_search(query.to_string(), Some(limit.min(10)))
            .await
            .map_err(ConnectorError::from)?;
        Ok(result
            .items
            .into_iter()
            .map(|item| {
                let url = (!item.url.trim().is_empty()).then_some(item.url.clone());
                let locator = url.clone().unwrap_or_else(|| item.content_id.clone());
                ContentRef {
                    locator,
                    title: item.title,
                    snippet: item.content_text.chars().take(500).collect(),
                    url,
                }
            })
            .collect())
    }

    async fn fetch(&self, locator: &str) -> Result<FetchedContent, ConnectorError> {
        if !locator.starts_with("http://") && !locator.starts_with("https://") {
            return Err(ConnectorError::Unsupported);
        }
        let fetched = super::web_url::web_url_fetch(locator.to_string())
            .await
            .map_err(ConnectorError::from)?;
        Ok(FetchedContent {
            url: Some(fetched.url),
            title: fetched.title,
            author: None,
            text: fetched.text,
        })
    }

    fn to_source(&self, content: &FetchedContent, selection: Option<&str>) -> SourceCaptureInput {
        SourceCaptureInput {
            platform: "zhihu".into(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zhihu_source_keeps_legacy_platform_and_generic_origin() {
        let connector = ZhihuConnector;
        let content = FetchedContent {
            url: Some("https://www.zhihu.com/question/1/answer/2".into()),
            title: Some("示例".into()),
            author: Some("作者".into()),
            text: "正文".into(),
        };
        let source = connector.to_source(&content, Some("选段"));
        assert_eq!(source.platform, "zhihu");
        assert_eq!(source.source_kind.as_deref(), Some("web"));
        assert_eq!(source.connector_id.as_deref(), Some("zhihu"));
        assert_eq!(source.selected_text, "选段");
    }
}
