//! HTTP client types for WASM platform.

use crate::{HeaderValue, HttpClient, HttpFuture, Request, Response};
use futures::io::AsyncRead;
use std::{
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// A URL wrapper for WASM.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Url(String);

impl Url {
    /// Parse a URL from a string.
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        Ok(Self(s.to_string()))
    }

    /// Get the URL as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for Url {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An async body for HTTP requests/responses.
pub enum AsyncBody {
    /// An empty body.
    Empty,
    /// A body containing bytes.
    Bytes(Vec<u8>),
}

impl Default for AsyncBody {
    fn default() -> Self {
        Self::Empty
    }
}

impl From<()> for AsyncBody {
    fn from(_: ()) -> Self {
        Self::Empty
    }
}

impl From<Vec<u8>> for AsyncBody {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }
}

impl From<String> for AsyncBody {
    fn from(s: String) -> Self {
        Self::Bytes(s.into_bytes())
    }
}

impl From<&str> for AsyncBody {
    fn from(s: &str) -> Self {
        Self::Bytes(s.as_bytes().to_vec())
    }
}

impl AsyncRead for AsyncBody {
    fn poll_read(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = self.get_mut();
        match this {
            AsyncBody::Empty => Poll::Ready(Ok(0)),
            AsyncBody::Bytes(bytes) => {
                let len = bytes.len().min(buf.len());
                buf[..len].copy_from_slice(&bytes[..len]);
                *bytes = bytes[len..].to_vec();
                Poll::Ready(Ok(len))
            }
        }
    }
}

/// A real HTTP client for WASM using the fetch API.
pub struct WebHttpClient {
    user_agent: Option<HeaderValue>,
}

impl WebHttpClient {
    /// Creates a new WebHttpClient.
    pub fn new() -> Arc<dyn HttpClient> {
        Arc::new(Self {
            user_agent: HeaderValue::from_str("gpui-web").ok(),
        })
    }

    /// Creates a new WebHttpClient with a custom user agent.
    pub fn with_user_agent(user_agent: &str) -> Arc<dyn HttpClient> {
        Arc::new(Self {
            user_agent: HeaderValue::from_str(user_agent).ok(),
        })
    }
}

impl Default for WebHttpClient {
    fn default() -> Self {
        Self {
            user_agent: HeaderValue::from_str("gpui-web").ok(),
        }
    }
}

impl HttpClient for WebHttpClient {
    fn user_agent(&self) -> Option<&HeaderValue> {
        self.user_agent.as_ref()
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn send(&self, req: Request<AsyncBody>) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let user_agent = self.user_agent.clone();
        Box::pin(async move {
            let window = web_sys::window()
                .ok_or_else(|| anyhow::anyhow!("No window object available"))?;

            let uri = req.uri().to_string();
            let method = req.method().as_str();

            let opts = web_sys::RequestInit::new();
            opts.set_method(method);

            // Set request body if present
            let body = req.into_body();
            match body {
                AsyncBody::Empty => {}
                AsyncBody::Bytes(bytes) => {
                    let uint8_array = js_sys::Uint8Array::from(&bytes[..]);
                    opts.set_body(&uint8_array);
                }
            }

            // Create headers
            let headers = web_sys::Headers::new()
                .map_err(|e| anyhow::anyhow!("Failed to create headers: {:?}", e))?;

            // Add user agent if present
            if let Some(ua) = user_agent {
                if let Ok(ua_str) = ua.to_str() {
                    headers.set("User-Agent", ua_str).ok();
                }
            }

            opts.set_headers(&headers);

            let request = web_sys::Request::new_with_str_and_init(&uri, &opts)
                .map_err(|e| anyhow::anyhow!("Failed to create request: {:?}", e))?;

            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| anyhow::anyhow!("Fetch failed: {:?}", e))?;

            let resp: web_sys::Response = resp_value
                .dyn_into()
                .map_err(|_| anyhow::anyhow!("Response is not a Response object"))?;

            let status = http::StatusCode::from_u16(resp.status())
                .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);

            let body_bytes = if let Ok(array_buffer_promise) = resp.array_buffer() {
                let array_buffer = JsFuture::from(array_buffer_promise)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get response body: {:?}", e))?;
                let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                uint8_array.to_vec()
            } else {
                Vec::new()
            };

            let response_body = if body_bytes.is_empty() {
                AsyncBody::Empty
            } else {
                AsyncBody::Bytes(body_bytes)
            };

            Response::builder()
                .status(status)
                .body(response_body)
                .map_err(|e| anyhow::anyhow!("Failed to build response: {}", e))
        })
    }

    fn get(
        &self,
        uri: &str,
        _body: AsyncBody,
        _follow_redirects: bool,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let user_agent = self.user_agent.clone();
        let uri = uri.to_string();
        Box::pin(async move {
            let window = web_sys::window()
                .ok_or_else(|| anyhow::anyhow!("No window object available"))?;

            let opts = web_sys::RequestInit::new();
            opts.set_method("GET");

            // Create headers
            let headers = web_sys::Headers::new()
                .map_err(|e| anyhow::anyhow!("Failed to create headers: {:?}", e))?;

            if let Some(ua) = user_agent {
                if let Ok(ua_str) = ua.to_str() {
                    headers.set("User-Agent", ua_str).ok();
                }
            }

            opts.set_headers(&headers);

            let request = web_sys::Request::new_with_str_and_init(&uri, &opts)
                .map_err(|e| anyhow::anyhow!("Failed to create request: {:?}", e))?;

            let resp_value = JsFuture::from(window.fetch_with_request(&request))
                .await
                .map_err(|e| anyhow::anyhow!("Fetch failed: {:?}", e))?;

            let resp: web_sys::Response = resp_value
                .dyn_into()
                .map_err(|_| anyhow::anyhow!("Response is not a Response object"))?;

            let status = http::StatusCode::from_u16(resp.status())
                .unwrap_or(http::StatusCode::INTERNAL_SERVER_ERROR);

            let body_bytes = if let Ok(array_buffer_promise) = resp.array_buffer() {
                let array_buffer = JsFuture::from(array_buffer_promise)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get response body: {:?}", e))?;
                let uint8_array = js_sys::Uint8Array::new(&array_buffer);
                uint8_array.to_vec()
            } else {
                Vec::new()
            };

            let response_body = if body_bytes.is_empty() {
                AsyncBody::Empty
            } else {
                AsyncBody::Bytes(body_bytes)
            };

            Response::builder()
                .status(status)
                .body(response_body)
                .map_err(|e| anyhow::anyhow!("Failed to build response: {}", e))
        })
    }
}

/// A fake HTTP client for testing on WASM.
pub struct FakeHttpClient;

impl FakeHttpClient {
    /// Creates a fake HTTP client that returns 404 responses.
    pub fn with_404_response() -> Arc<dyn HttpClient> {
        Arc::new(Self)
    }
}

impl HttpClient for FakeHttpClient {
    fn user_agent(&self) -> Option<&HeaderValue> {
        None
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn send(&self, _req: Request<AsyncBody>) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        Box::pin(async move {
            anyhow::bail!("FakeHttpClient: no HTTP client available in WASM")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::io::AsyncReadExt;

    #[test]
    fn test_url_parse() {
        let url = Url::parse("https://example.com").unwrap();
        assert_eq!(url.as_str(), "https://example.com");
        assert_eq!(url.to_string(), "https://example.com");
    }

    #[test]
    fn test_url_equality() {
        let url1 = Url::parse("https://example.com").unwrap();
        let url2 = Url::parse("https://example.com").unwrap();
        let url3 = Url::parse("https://other.com").unwrap();
        assert_eq!(url1, url2);
        assert_ne!(url1, url3);
    }

    #[test]
    fn test_async_body_default() {
        let body = AsyncBody::default();
        assert!(matches!(body, AsyncBody::Empty));
    }

    #[test]
    fn test_async_body_from_unit() {
        let body: AsyncBody = ().into();
        assert!(matches!(body, AsyncBody::Empty));
    }

    #[test]
    fn test_async_body_from_vec() {
        let data = vec![1, 2, 3, 4];
        let body: AsyncBody = data.clone().into();
        assert!(matches!(body, AsyncBody::Bytes(bytes) if bytes == data));
    }

    #[test]
    fn test_async_body_from_string() {
        let s = "hello world".to_string();
        let body: AsyncBody = s.clone().into();
        assert!(matches!(body, AsyncBody::Bytes(bytes) if bytes == s.as_bytes()));
    }

    #[test]
    fn test_async_body_from_str() {
        let s = "hello world";
        let body: AsyncBody = s.into();
        assert!(matches!(body, AsyncBody::Bytes(bytes) if bytes == s.as_bytes()));
    }

    #[test]
    fn test_fake_http_client_no_user_agent() {
        let client = FakeHttpClient::with_404_response();
        assert!(client.user_agent().is_none());
    }

    #[test]
    fn test_fake_http_client_no_proxy() {
        let client = FakeHttpClient::with_404_response();
        assert!(client.proxy().is_none());
    }
}
