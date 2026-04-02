/*
 * SPDX-FileCopyrightText: 2020 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 Gervasio Marchand <github@gervas.io>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

#![allow(clippy::significant_drop_tightening)]

use std::sync::LazyLock;

use crate::link_validator::LinkCheckResult;

use log::debug;
use mle::WildMatch;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::StatusCode;
use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;

const BROWSER_ACCEPT_HEADER: &str =
    "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8";

pub async fn check_http(
    target: &url::Url,
    do_not_warn_for_redirect_to: &[WildMatch],
) -> LinkCheckResult {
    debug!("Checking http link target '{target:?}' ...");
    match http_request(target, do_not_warn_for_redirect_to).await {
        Ok(response) => response,
        Err(error_msg) => LinkCheckResult::Failed(format!("Http(s) request failed: {error_msg}")),
    }
}

fn new_request(method: Method, url: &reqwest::Url) -> Request {
    let mut req = Request::new(method, url.clone());
    let headers = req.headers_mut();
    headers.insert(ACCEPT, BROWSER_ACCEPT_HEADER.parse().unwrap());
    headers.insert(USER_AGENT, "mlc (github.com/becheran/mlc)".parse().unwrap());
    req
}

async fn http_request(
    url: &reqwest::Url,
    do_not_warn_for_redirect_to: &[WildMatch],
) -> reqwest::Result<LinkCheckResult> {
    static CLIENT: LazyLock<Client> = LazyLock::new(|| {
        reqwest::Client::builder()
            .brotli(true)
            .gzip(true)
            .deflate(true)
            .build()
            .expect("Bug! failed to build client")
    });

    fn status_to_string(status: StatusCode) -> String {
        format!(
            "{} - {}",
            status.as_str(),
            status.canonical_reason().unwrap_or("Unknown reason")
        )
    }

    let check_redirect = |response_url: &reqwest::Url| -> reqwest::Result<LinkCheckResult> {
        if response_url == url
            || do_not_warn_for_redirect_to
                .iter()
                .any(|x| x.matches(response_url.as_ref()))
        {
            Ok(LinkCheckResult::Ok)
        } else {
            Ok(LinkCheckResult::Warning(
                "Request was redirected to ".to_string() + response_url.as_ref(),
            ))
        }
    };

    let head_request = new_request(Method::HEAD, url);
    let get_request = new_request(Method::GET, url);

    let response = match CLIENT.execute(head_request).await {
        Ok(r) => r,
        Err(e) => {
            println!("Head request error: {e}. Retry with get-request.");
            CLIENT.execute(get_request).await?
        }
    };

    let status = response.status();
    if status.is_success() || status.is_redirection() {
        check_redirect(response.url())
    } else if status.is_redirection() {
        // Only if > 10 redirects
        Ok(LinkCheckResult::Warning(status_to_string(status)))
    } else {
        debug!("Got the status code {status:?}. Retry with get-request.");
        let get_request = Request::new(Method::GET, url.clone());

        let response = CLIENT.execute(get_request).await?;
        let status = response.status();
        if status.is_success() || status.is_redirection() {
            check_redirect(response.url())
        } else {
            Ok(LinkCheckResult::Failed(status_to_string(status)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    async fn check_http_str(
        url_str: &str,
        do_not_warn_for_redirect_to: &[WildMatch],
    ) -> LinkCheckResult {
        match reqwest::Url::parse(url_str) {
            Ok(url) => {
                println!("{url:#?}");
                check_http(&url, do_not_warn_for_redirect_to).await
            }
            Err(err) => {
                LinkCheckResult::Failed(format!("Failed to parse '{url_str}' as (HTTP-)URL: {err}"))
            }
        }
    }

    #[tokio::test]
    async fn check_http_is_available() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;

        let result = check_http_str(&server.url(), &[]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_fail() {
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(500)
            .create_async()
            .await;

        let result = check_http_str(&server.url(), &[]).await;
        assert_eq!(
            result,
            LinkCheckResult::Failed("500 - Internal Server Error".to_string())
        );
    }

    #[tokio::test]
    async fn check_http_is_redirection() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;

        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http_str(&server.url(), &[]).await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(format!(
                "Request was redirected to {}/",
                &redirect_server.url()
            ))
        );
    }

    #[tokio::test]
    async fn check_http_redirection_do_not_warn_if_ignored() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http_str(
            &server.url(),
            &[WildMatch::new(&format!("{}*", &redirect_server.url()))],
        )
        .await;

        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_redirection_do_not_warn_if_ignored_star_pattern() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http_str(&server.url(), &[WildMatch::new("*")]).await;

        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_redirection_do_warn_if_ignored_mismatch() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(200)
            .create_async()
            .await;
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http_str(
            &server.url(),
            &[WildMatch::new("http://is-mismatched.com/*")],
        )
        .await;

        assert_eq!(
            result,
            LinkCheckResult::Warning(format!(
                "Request was redirected to {}/",
                &redirect_server.url()
            ))
        );
    }

    #[tokio::test]
    async fn check_http_is_redirection_failure() {
        let mut redirect_server = mockito::Server::new_async().await;
        redirect_server
            .mock("GET", "/")
            .with_status(403)
            .create_async()
            .await;
        let mut server = mockito::Server::new_async().await;
        server
            .mock("GET", "/")
            .with_status(301)
            .with_header("Location", &redirect_server.url())
            .create_async()
            .await;

        let result = check_http_str(&server.url(), &[]).await;

        assert_eq!(
            result,
            LinkCheckResult::Failed("403 - Forbidden".to_string())
        );
    }

    //#[tokio::test]
    async fn check_http_request_with_hash() {
        let result = check_http_str("https://gitlab.com/becheran/mlc#bla", &[]).await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    //#[tokio::test]
    async fn check_http_request_redirection_with_hash() {
        let result = check_http_str("http://gitlab.com/becheran/mlc#bla", &[]).await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(
                "Request was redirected to https://gitlab.com/becheran/mlc".to_string()
            )
        );
    }

    //#[tokio::test]
    async fn check_wrong_http_request() {
        let result = check_http_str("https://doesNotExist.me/even/less/likelly", &[]).await;
        assert!(result != LinkCheckResult::Ok);
    }
}
