//! Pagination helpers.

use futures::{stream, Stream};
use serde::de::DeserializeOwned;

use crate::{Client, Result};

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
}

impl<T> Page<T>
where
    T: Clone + DeserializeOwned + Send + 'static,
{
    /// Attaches client context and returns self.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = Some(client);
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

        let next: Page<T> = client.get_json(path).await?;
        Ok(Some(next.with_client(client)))
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
    pub has_more: bool,
    /// Cursor for first item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_id: Option<String>,
    /// Cursor for last item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_id: Option<String>,
    #[serde(skip)]
    next_page_path: Option<String>,
    #[serde(skip)]
    client: Option<Client>,
}

impl<T> CursorPage<T>
where
    T: Clone + DeserializeOwned + Send + 'static,
{
    /// Attaches client and next-page path context.
    pub fn with_context(mut self, client: Client, next_page_path: Option<String>) -> Self {
        self.client = Some(client);
        self.next_page_path = next_page_path;
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
        if !self.has_more {
            return Ok(None);
        }

        let Some(path) = self.next_page_path.as_deref() else {
            return Ok(None);
        };
        let Some(client) = self.client.clone() else {
            return Ok(None);
        };

        let next: CursorPage<T> = client.get_json(path).await?;
        Ok(Some(next.with_context(client, None)))
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
    use super::{CursorPage, Page};

    #[tokio::test]
    async fn next_page_is_none_without_context() {
        let page = Page::<u8> {
            data: vec![1],
            object: "list".to_owned(),
            next_page: Some("/foo".to_owned()),
            client: None,
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
            next_page_path: Some("/v1/models?after=b".to_owned()),
            client: None,
        };

        assert!(page
            .next_page()
            .await
            .expect("no transport error")
            .is_none());
    }
}
