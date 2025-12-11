#[cfg(not(target_arch = "wasm32"))]
mod async_body;
#[cfg(not(target_arch = "wasm32"))]
pub mod github;
#[cfg(not(target_arch = "wasm32"))]
pub mod github_download;
#[cfg(target_arch = "wasm32")]
mod web;

pub use anyhow::{Result, anyhow};
#[cfg(not(target_arch = "wasm32"))]
pub use async_body::{AsyncBody, Inner};
#[cfg(target_arch = "wasm32")]
pub use web::{AsyncBody, WebHttpClient};
#[cfg(not(target_arch = "wasm32"))]
use derive_more::Deref;
use http::HeaderValue;
pub use http::{self, Method, Request, Response, StatusCode, Uri, request::Builder};

#[cfg(not(target_arch = "wasm32"))]
use futures::{FutureExt as _, future};
#[cfg(target_arch = "wasm32")]
use futures::future::LocalBoxFuture;
#[cfg(not(target_arch = "wasm32"))]
use parking_lot::Mutex;
#[cfg(not(target_arch = "wasm32"))]
use serde::Serialize;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(all(feature = "test-support", not(target_arch = "wasm32")))]
use std::{any::type_name, fmt};
#[cfg(not(target_arch = "wasm32"))]
pub use url::{Host, Url};
#[cfg(target_arch = "wasm32")]
pub use web::Url;

/// Type alias for HTTP futures - Send on native, non-Send on WASM
#[cfg(not(target_arch = "wasm32"))]
pub type HttpFuture<'a, T> = futures::future::BoxFuture<'a, T>;
#[cfg(target_arch = "wasm32")]
pub type HttpFuture<'a, T> = LocalBoxFuture<'a, T>;

#[derive(Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RedirectPolicy {
    #[default]
    NoFollow,
    FollowLimit(u32),
    FollowAll,
}
pub struct FollowRedirects(pub bool);

pub trait HttpRequestExt {
    /// Conditionally modify self with the given closure.
    fn when(self, condition: bool, then: impl FnOnce(Self) -> Self) -> Self
    where
        Self: Sized,
    {
        if condition { then(self) } else { self }
    }

    /// Conditionally unwrap and modify self with the given closure, if the given option is Some.
    fn when_some<T>(self, option: Option<T>, then: impl FnOnce(Self, T) -> Self) -> Self
    where
        Self: Sized,
    {
        match option {
            Some(value) => then(self, value),
            None => self,
        }
    }

    /// Whether or not to follow redirects
    fn follow_redirects(self, follow: RedirectPolicy) -> Self;
}

impl HttpRequestExt for http::request::Builder {
    fn follow_redirects(self, follow: RedirectPolicy) -> Self {
        self.extension(follow)
    }
}

/// HTTP client trait for making HTTP requests.
/// On native platforms, this requires Send + Sync. On WASM, these bounds are not required.
#[cfg(not(target_arch = "wasm32"))]
pub trait HttpClient: 'static + Send + Sync {
    fn user_agent(&self) -> Option<&HeaderValue>;

    fn proxy(&self) -> Option<&Url>;

    fn send(
        &self,
        req: http::Request<AsyncBody>,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>>;

    fn get(
        &self,
        uri: &str,
        body: AsyncBody,
        follow_redirects: bool,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let request = Builder::new()
            .uri(uri)
            .follow_redirects(if follow_redirects {
                RedirectPolicy::FollowAll
            } else {
                RedirectPolicy::NoFollow
            })
            .body(body);

        match request {
            Ok(request) => self.send(request),
            Err(e) => Box::pin(async move { Err(e.into()) }),
        }
    }

    fn post_json(
        &self,
        uri: &str,
        body: AsyncBody,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let request = Builder::new()
            .uri(uri)
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(body);

        match request {
            Ok(request) => self.send(request),
            Err(e) => Box::pin(async move { Err(e.into()) }),
        }
    }

    #[cfg(feature = "test-support")]
    fn as_fake(&self) -> &FakeHttpClient {
        panic!("called as_fake on {}", type_name::<Self>())
    }

    fn send_multipart_form<'a>(
        &'a self,
        _url: &str,
        _request: reqwest::multipart::Form,
    ) -> HttpFuture<'a, anyhow::Result<Response<AsyncBody>>> {
        future::ready(Err(anyhow!("not implemented"))).boxed()
    }
}

/// HTTP client trait for WASM - no Send + Sync bounds required
#[cfg(target_arch = "wasm32")]
pub trait HttpClient: 'static {
    fn user_agent(&self) -> Option<&HeaderValue>;

    fn proxy(&self) -> Option<&Url>;

    fn send(
        &self,
        req: http::Request<AsyncBody>,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>>;

    fn get(
        &self,
        uri: &str,
        body: AsyncBody,
        follow_redirects: bool,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let request = Builder::new()
            .uri(uri)
            .follow_redirects(if follow_redirects {
                RedirectPolicy::FollowAll
            } else {
                RedirectPolicy::NoFollow
            })
            .body(body);

        match request {
            Ok(request) => self.send(request),
            Err(e) => Box::pin(async move { Err(e.into()) }),
        }
    }

    fn post_json(
        &self,
        uri: &str,
        body: AsyncBody,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let request = Builder::new()
            .uri(uri)
            .method(Method::POST)
            .header("Content-Type", "application/json")
            .body(body);

        match request {
            Ok(request) => self.send(request),
            Err(e) => Box::pin(async move { Err(e.into()) }),
        }
    }
}

// ============================================================================
// Native-only types below (not available on WASM)
// ============================================================================

/// An [`HttpClient`] that may have a proxy.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Deref)]
pub struct HttpClientWithProxy {
    #[deref]
    client: Arc<dyn HttpClient>,
    proxy: Option<Url>,
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpClientWithProxy {
    /// Returns a new [`HttpClientWithProxy`] with the given proxy URL.
    pub fn new(client: Arc<dyn HttpClient>, proxy_url: Option<String>) -> Self {
        let proxy_url = proxy_url
            .and_then(|proxy| proxy.parse().ok())
            .or_else(read_proxy_from_env);

        Self::new_url(client, proxy_url)
    }
    pub fn new_url(client: Arc<dyn HttpClient>, proxy_url: Option<Url>) -> Self {
        Self {
            client,
            proxy: proxy_url,
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpClient for HttpClientWithProxy {
    fn send(
        &self,
        req: Request<AsyncBody>,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        self.client.send(req)
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        self.client.user_agent()
    }

    fn proxy(&self) -> Option<&Url> {
        self.proxy.as_ref()
    }

    #[cfg(feature = "test-support")]
    fn as_fake(&self) -> &FakeHttpClient {
        self.client.as_fake()
    }

    fn send_multipart_form<'a>(
        &'a self,
        url: &str,
        form: reqwest::multipart::Form,
    ) -> HttpFuture<'a, anyhow::Result<Response<AsyncBody>>> {
        self.client.send_multipart_form(url, form)
    }
}

/// An [`HttpClient`] that has a base URL.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Deref)]
pub struct HttpClientWithUrl {
    base_url: Mutex<String>,
    #[deref]
    client: HttpClientWithProxy,
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpClientWithUrl {
    /// Returns a new [`HttpClientWithUrl`] with the given base URL.
    pub fn new(
        client: Arc<dyn HttpClient>,
        base_url: impl Into<String>,
        proxy_url: Option<String>,
    ) -> Self {
        let client = HttpClientWithProxy::new(client, proxy_url);

        Self {
            base_url: Mutex::new(base_url.into()),
            client,
        }
    }

    pub fn new_url(
        client: Arc<dyn HttpClient>,
        base_url: impl Into<String>,
        proxy_url: Option<Url>,
    ) -> Self {
        let client = HttpClientWithProxy::new_url(client, proxy_url);

        Self {
            base_url: Mutex::new(base_url.into()),
            client,
        }
    }

    /// Returns the base URL.
    pub fn base_url(&self) -> String {
        self.base_url.lock().clone()
    }

    /// Sets the base URL.
    pub fn set_base_url(&self, base_url: impl Into<String>) {
        let base_url = base_url.into();
        *self.base_url.lock() = base_url;
    }

    /// Builds a URL using the given path.
    pub fn build_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url(), path)
    }

    /// Builds a Zed API URL using the given path.
    pub fn build_zed_api_url(&self, path: &str, query: &[(&str, &str)]) -> Result<Url> {
        let base_url = self.base_url();
        let base_api_url = match base_url.as_ref() {
            "https://zed.dev" => "https://api.zed.dev",
            "https://staging.zed.dev" => "https://api-staging.zed.dev",
            "http://localhost:3000" => "http://localhost:8080",
            other => other,
        };

        Ok(Url::parse_with_params(
            &format!("{}{}", base_api_url, path),
            query,
        )?)
    }

    /// Builds a Zed Cloud URL using the given path.
    pub fn build_zed_cloud_url(&self, path: &str) -> Result<Url> {
        let base_url = self.base_url();
        let base_api_url = match base_url.as_ref() {
            "https://zed.dev" => "https://cloud.zed.dev",
            "https://staging.zed.dev" => "https://cloud.zed.dev",
            "http://localhost:3000" => "http://localhost:8787",
            other => other,
        };

        Ok(Url::parse(&format!("{}{}", base_api_url, path))?)
    }

    /// Builds a Zed Cloud URL using the given path and query params.
    pub fn build_zed_cloud_url_with_query(&self, path: &str, query: impl Serialize) -> Result<Url> {
        let base_url = self.base_url();
        let base_api_url = match base_url.as_ref() {
            "https://zed.dev" => "https://cloud.zed.dev",
            "https://staging.zed.dev" => "https://cloud.zed.dev",
            "http://localhost:3000" => "http://localhost:8787",
            other => other,
        };
        let query = serde_urlencoded::to_string(&query)?;
        Ok(Url::parse(&format!("{}{}?{}", base_api_url, path, query))?)
    }

    /// Builds a Zed LLM URL using the given path.
    pub fn build_zed_llm_url(&self, path: &str, query: &[(&str, &str)]) -> Result<Url> {
        let base_url = self.base_url();
        let base_api_url = match base_url.as_ref() {
            "https://zed.dev" => "https://cloud.zed.dev",
            "https://staging.zed.dev" => "https://llm-staging.zed.dev",
            "http://localhost:3000" => "http://localhost:8787",
            other => other,
        };

        Ok(Url::parse_with_params(
            &format!("{}{}", base_api_url, path),
            query,
        )?)
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpClient for HttpClientWithUrl {
    fn send(
        &self,
        req: Request<AsyncBody>,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        self.client.send(req)
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        self.client.user_agent()
    }

    fn proxy(&self) -> Option<&Url> {
        self.client.proxy.as_ref()
    }

    #[cfg(feature = "test-support")]
    fn as_fake(&self) -> &FakeHttpClient {
        self.client.as_fake()
    }

    fn send_multipart_form<'a>(
        &'a self,
        url: &str,
        request: reqwest::multipart::Form,
    ) -> HttpFuture<'a, anyhow::Result<Response<AsyncBody>>> {
        self.client.send_multipart_form(url, request)
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_proxy_from_env() -> Option<Url> {
    const ENV_VARS: &[&str] = &[
        "ALL_PROXY",
        "all_proxy",
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
    ];

    ENV_VARS
        .iter()
        .find_map(|var| std::env::var(var).ok())
        .and_then(|env| env.parse().ok())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn read_no_proxy_from_env() -> Option<String> {
    const ENV_VARS: &[&str] = &["NO_PROXY", "no_proxy"];

    ENV_VARS.iter().find_map(|var| std::env::var(var).ok())
}

#[cfg(not(target_arch = "wasm32"))]
pub struct BlockedHttpClient;

#[cfg(not(target_arch = "wasm32"))]
impl BlockedHttpClient {
    pub fn new() -> Self {
        BlockedHttpClient
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl HttpClient for BlockedHttpClient {
    fn send(
        &self,
        _req: Request<AsyncBody>,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        Box::pin(async {
            Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "BlockedHttpClient disallowed request",
            )
            .into())
        })
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        None
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    #[cfg(feature = "test-support")]
    fn as_fake(&self) -> &FakeHttpClient {
        panic!("called as_fake on {}", type_name::<Self>())
    }
}

#[cfg(all(feature = "test-support", not(target_arch = "wasm32")))]
type FakeHttpHandler = Arc<
    dyn Fn(Request<AsyncBody>) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>>
        + Send
        + Sync
        + 'static,
>;

#[cfg(all(feature = "test-support", not(target_arch = "wasm32")))]
pub struct FakeHttpClient {
    handler: Mutex<Option<FakeHttpHandler>>,
    user_agent: HeaderValue,
}

#[cfg(all(feature = "test-support", not(target_arch = "wasm32")))]
impl FakeHttpClient {
    pub fn create<Fut, F>(handler: F) -> Arc<HttpClientWithUrl>
    where
        Fut: futures::Future<Output = anyhow::Result<Response<AsyncBody>>> + Send + 'static,
        F: Fn(Request<AsyncBody>) -> Fut + Send + Sync + 'static,
    {
        Arc::new(HttpClientWithUrl {
            base_url: Mutex::new("http://test.example".into()),
            client: HttpClientWithProxy {
                client: Arc::new(Self {
                    handler: Mutex::new(Some(Arc::new(move |req| Box::pin(handler(req))))),
                    user_agent: HeaderValue::from_static(type_name::<Self>()),
                }),
                proxy: None,
            },
        })
    }

    pub fn with_404_response() -> Arc<HttpClientWithUrl> {
        log::warn!("Using fake HTTP client with 404 response");
        Self::create(|_| async move {
            Ok(Response::builder()
                .status(404)
                .body(Default::default())
                .unwrap())
        })
    }

    pub fn with_200_response() -> Arc<HttpClientWithUrl> {
        log::warn!("Using fake HTTP client with 200 response");
        Self::create(|_| async move {
            Ok(Response::builder()
                .status(200)
                .body(Default::default())
                .unwrap())
        })
    }

    pub fn replace_handler<Fut, F>(&self, new_handler: F)
    where
        Fut: futures::Future<Output = anyhow::Result<Response<AsyncBody>>> + Send + 'static,
        F: Fn(FakeHttpHandler, Request<AsyncBody>) -> Fut + Send + Sync + 'static,
    {
        let mut handler = self.handler.lock();
        let old_handler = handler.take().unwrap();
        *handler = Some(Arc::new(move |req| {
            Box::pin(new_handler(old_handler.clone(), req))
        }));
    }
}

#[cfg(all(feature = "test-support", not(target_arch = "wasm32")))]
impl fmt::Debug for FakeHttpClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FakeHttpClient").finish()
    }
}

#[cfg(all(feature = "test-support", not(target_arch = "wasm32")))]
impl HttpClient for FakeHttpClient {
    fn send(
        &self,
        req: Request<AsyncBody>,
    ) -> HttpFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        ((self.handler.lock().as_ref().unwrap())(req)) as _
    }

    fn user_agent(&self) -> Option<&HeaderValue> {
        Some(&self.user_agent)
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }

    fn as_fake(&self) -> &FakeHttpClient {
        self
    }
}
