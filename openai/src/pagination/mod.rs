//! Pagination helpers.

use futures::{stream, Stream};
use serde::{de::DeserializeOwned, Serialize};

use crate::{config::RequestOptions, error::Error, Client, Result};

/// Basic list page container.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Page<T> {
    /// List items.
    pub data: Vec<T>,
    /// Object type.
    pub object: String,
    /// Optional next page URL/path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page: Option<String>,
    #[serde(skip)]
    client: Option<Client>,
    #[serde(skip)]
    options: RequestOptions,
}

impl<T> Page<T>
where
    T: Clone + DeserializeOwned + Serialize + Send + 'static,
{
    /// Attaches client context and returns self.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    /// Attaches client context and request options, then returns self.
    pub(crate) fn with_request_options(mut self, client: Client, options: RequestOptions) -> Self {
        self.client = Some(client);
        self.options = options;
        self
    }

    /// Returns true when this page has no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns true when a follow-up page path is present.
    #[must_use]
    pub fn has_next_page(&self) -> bool {
        self.next_page.is_some()
    }

    /// Fetches next page when available.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn next_page(&self) -> Result<Option<Page<T>>> {
        let Some(path) = self.next_page.as_deref() else {
            return Ok(None);
        };
        let Some(client) = self.client.clone() else {
            return Ok(None);
        };

        let next: Page<T> = client.get_json_with_options(path, &self.options).await?;
        Ok(Some(
            next.with_request_options(client, self.options.clone()),
        ))
    }

    /// Converts all pages into an item stream.
    pub fn into_stream(self) -> impl Stream<Item = Result<T>> {
        stream::try_unfold(Some((self, 0_usize)), |state| async move {
            let Some((page, index)) = state else {
                return Ok(None);
            };

            if let Some(item) = page.data.get(index).cloned() {
                return Ok(Some((item, Some((page, index + 1)))));
            }

            let mut next = page.next_page().await?;
            while let Some(np) = next {
                if let Some(item) = np.data.first().cloned() {
                    return Ok(Some((item, Some((np, 1_usize)))));
                }
                next = np.next_page().await?;
            }

            Ok(None)
        })
    }
}

/// Cursor-based page container.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct CursorPage<T> {
    /// List items.
    pub data: Vec<T>,
    /// Object type.
    pub object: String,
    /// Whether additional pages exist.
    #[serde(default)]
    pub has_more: bool,
    /// Cursor for first item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    /// Cursor for last item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    #[serde(skip)]
    base_path: Option<String>,
    #[serde(skip)]
    client: Option<Client>,
    #[serde(skip)]
    options: RequestOptions,
}

impl<T> CursorPage<T>
where
    T: Clone + DeserializeOwned + Serialize + Send + 'static,
{
    /// Attaches client and request path context for fetching subsequent pages.
    pub fn with_context(mut self, client: Client, base_path: impl Into<String>) -> Self {
        self.client = Some(client);
        self.base_path = Some(base_path.into());
        self
    }

    /// Attaches client, request path, and request options context.
    pub(crate) fn with_request_options(
        mut self,
        client: Client,
        base_path: impl Into<String>,
        options: RequestOptions,
    ) -> Self {
        self.client = Some(client);
        self.base_path = Some(base_path.into());
        self.options = options;
        self
    }

    /// Returns true when this page has no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Fetches next cursor page when available.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn next_page(&self) -> Result<Option<CursorPage<T>>> {
        if !self.has_more || self.data.is_empty() {
            return Ok(None);
        }

        let Some(base_path) = self.base_path.as_deref() else {
            return Ok(None);
        };
        let Some(client) = self.client.clone() else {
            return Ok(None);
        };
        let Some(after) = self.last_id.clone().or_else(|| last_item_id(&self.data)) else {
            return Ok(None);
        };

        let path = path_with_replaced_query_param(base_path, "after", &after);
        let next: CursorPage<T> = client.get_json_with_options(&path, &self.options).await?;
        Ok(Some(next.with_request_options(
            client,
            base_path.to_owned(),
            self.options.clone(),
        )))
    }

    /// Converts all items across pages into a stream.
    pub fn into_stream(self) -> impl Stream<Item = Result<T>> {
        stream::try_unfold(Some((self, 0_usize)), |state| async move {
            let Some((page, index)) = state else {
                return Ok(None);
            };

            if let Some(item) = page.data.get(index).cloned() {
                return Ok(Some((item, Some((page, index + 1)))));
            }

            let mut next = page.next_page().await?;
            while let Some(np) = next {
                if let Some(item) = np.data.first().cloned() {
                    return Ok(Some((item, Some((np, 1_usize)))));
                }
                next = np.next_page().await?;
            }

            Ok(None)
        })
    }
}

/// Builds a request path with URL-encoded query parameters.
pub(crate) fn path_with_query<Q>(path: &str, query: &Q) -> Result<String>
where
    Q: Serialize + ?Sized,
{
    let query = serde_urlencoded::to_string(query).map_err(|err| Error::Config {
        field: "query",
        message: err.to_string(),
    })?;
    if query.is_empty() {
        Ok(path.to_owned())
    } else {
        Ok(format!("{path}?{query}"))
    }
}

/// Adds or replaces a query parameter in a relative endpoint path.
pub(crate) fn path_with_replaced_query_param(path: &str, key: &str, value: &str) -> String {
    let (base, query) = path.split_once('?').unwrap_or((path, ""));
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());

    for (existing_key, existing_value) in url::form_urlencoded::parse(query.as_bytes()) {
        if existing_key != key {
            serializer.append_pair(&existing_key, &existing_value);
        }
    }
    serializer.append_pair(key, value);

    let query = serializer.finish();
    if query.is_empty() {
        base.to_owned()
    } else {
        format!("{base}?{query}")
    }
}

fn last_item_id<T>(items: &[T]) -> Option<String>
where
    T: Serialize,
{
    let value = serde_json::to_value(items.last()?).ok()?;
    value
        .get("id")
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
}

/// Conversation cursor page container.
///
/// Similar to [`CursorPage`] but uses a `last_id` field returned by the server
/// for cursor progression (used by the Conversations and Videos APIs).
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct ConversationCursorPage<T> {
    /// List items.
    pub data: Vec<T>,
    /// Whether additional pages exist.
    pub has_more: bool,
    /// The cursor pointing to the last item in the current page.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    /// The base path to use for fetching the next page.
    #[serde(skip)]
    base_path: Option<String>,
    #[serde(skip)]
    client: Option<Client>,
    #[serde(skip)]
    options: RequestOptions,
}

impl<T> ConversationCursorPage<T>
where
    T: Clone + DeserializeOwned + Serialize + Send + 'static,
{
    /// Attaches client and base path context.
    pub fn with_context(mut self, client: Client, base_path: impl Into<String>) -> Self {
        self.client = Some(client);
        self.base_path = Some(base_path.into());
        self
    }

    /// Attaches client, request path, and request options context.
    pub(crate) fn with_request_options(
        mut self,
        client: Client,
        base_path: impl Into<String>,
        options: RequestOptions,
    ) -> Self {
        self.client = Some(client);
        self.base_path = Some(base_path.into());
        self.options = options;
        self
    }

    /// Returns true when this page has no items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Fetches next conversation cursor page when available.
    ///
    /// Uses the `last_id` returned by the current page as the `after` query
    /// parameter for the next request.
    ///
    /// # Errors
    /// Returns [`crate::Error`] for transport or API failures.
    pub async fn next_page(&self) -> Result<Option<ConversationCursorPage<T>>> {
        if !self.has_more {
            return Ok(None);
        }

        let Some(last_id) = self.last_id.as_deref() else {
            return Ok(None);
        };
        if last_id.is_empty() {
            return Ok(None);
        }

        let Some(base_path) = self.base_path.as_deref() else {
            return Ok(None);
        };
        let Some(client) = self.client.clone() else {
            return Ok(None);
        };

        let separator = if base_path.contains('?') { "&" } else { "?" };
        let next_path = format!("{base_path}{separator}after={last_id}");

        let next: ConversationCursorPage<T> = client
            .get_json_with_options(&next_path, &self.options)
            .await?;
        Ok(Some(next.with_request_options(
            client,
            base_path.to_owned(),
            self.options.clone(),
        )))
    }

    /// Converts all items across pages into a stream.
    pub fn into_stream(self) -> impl Stream<Item = Result<T>> {
        stream::try_unfold(Some((self, 0_usize)), |state| async move {
            let Some((page, index)) = state else {
                return Ok(None);
            };

            if let Some(item) = page.data.get(index).cloned() {
                return Ok(Some((item, Some((page, index + 1)))));
            }

            let mut next = page.next_page().await?;
            while let Some(np) = next {
                if let Some(item) = np.data.first().cloned() {
                    return Ok(Some((item, Some((np, 1_usize)))));
                }
                next = np.next_page().await?;
            }

            Ok(None)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{path_with_replaced_query_param, ConversationCursorPage, CursorPage, Page};
    use crate::config::RequestOptions;

    #[tokio::test]
    async fn next_page_is_none_without_context() {
        let page = Page::<u8> {
            data: vec![1],
            object: "list".to_owned(),
            next_page: Some("/foo".to_owned()),
            client: None,
            options: RequestOptions::default(),
        };

        assert!(page
            .next_page()
            .await
            .expect("no transport error")
            .is_none());
    }

    #[test]
    fn page_helpers_report_empty_and_navigation_state() {
        let page = Page::<u8> {
            data: vec![],
            object: "list".to_owned(),
            next_page: Some("/v1/models?page=2".to_owned()),
            client: None,
            options: RequestOptions::default(),
        };

        assert!(page.is_empty());
        assert!(page.has_next_page());
    }

    #[tokio::test]
    async fn cursor_page_next_page_is_none_without_context() {
        let page = CursorPage::<u8> {
            data: vec![1],
            object: "list".to_owned(),
            has_more: true,
            first_id: Some("a".to_owned()),
            last_id: Some("b".to_owned()),
            base_path: Some("/v1/models?after=b".to_owned()),
            client: None,
            options: RequestOptions::default(),
        };

        assert!(page
            .next_page()
            .await
            .expect("no transport error")
            .is_none());
    }

    #[test]
    fn cursor_path_replaces_existing_after() {
        let path =
            path_with_replaced_query_param("/files?limit=2&after=file_old", "after", "file_new");

        assert_eq!(path, "/files?limit=2&after=file_new");
    }

    #[tokio::test]
    async fn conversation_cursor_page_next_page_is_none_without_context() {
        let page = ConversationCursorPage::<u8> {
            data: vec![1, 2],
            has_more: true,
            last_id: Some("item_abc".to_owned()),
            base_path: None,
            client: None,
            options: RequestOptions::default(),
        };

        assert!(page
            .next_page()
            .await
            .expect("no transport error")
            .is_none());
    }

    #[tokio::test]
    async fn conversation_cursor_page_no_more_returns_none() {
        let page = ConversationCursorPage::<u8> {
            data: vec![1],
            has_more: false,
            last_id: Some("item_abc".to_owned()),
            base_path: Some("/v1/conversations/conv_123/items".to_owned()),
            client: None,
            options: RequestOptions::default(),
        };

        assert!(page
            .next_page()
            .await
            .expect("no transport error")
            .is_none());
    }

    #[test]
    fn conversation_cursor_page_is_empty() {
        let page = ConversationCursorPage::<u8> {
            data: vec![],
            has_more: false,
            last_id: None,
            base_path: None,
            client: None,
            options: RequestOptions::default(),
        };
        assert!(page.is_empty());
    }

    #[tokio::test]
    async fn conversation_cursor_page_empty_last_id_returns_none() {
        let page = ConversationCursorPage::<u8> {
            data: vec![1],
            has_more: true,
            last_id: Some(String::new()),
            base_path: Some("/v1/items".to_owned()),
            client: None,
            options: RequestOptions::default(),
        };

        assert!(page
            .next_page()
            .await
            .expect("no transport error")
            .is_none());
    }
}
