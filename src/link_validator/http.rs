/*
 * SPDX-FileCopyrightText: 2020 - 2022 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2022 Gervasio Marchand <github@gervas.io>
 * SPDX-FileCopyrightText: 2022 - 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

use std::sync::LazyLock;

use crate::link_validator::LinkCheckResult;

use log::debug;
use reqwest::Client;
use reqwest::Method;
use reqwest::Request;
use reqwest::StatusCode;
use reqwest::header::ACCEPT;
use reqwest::header::USER_AGENT;

pub async fn check_http(target: &url::Url) -> LinkCheckResult {
    debug!("Checking http link target '{target:?}' ...");
    match http_request(target).await {
        Ok(response) => response,
        Err(error_msg) => LinkCheckResult::Failed(format!("Http(s) request failed: {error_msg}")),
    }
}

fn new_request(method: Method, url: &reqwest::Url) -> Request {
    let mut req = Request::new(method, url.clone());
    let headers = req.headers_mut();
    headers.insert(ACCEPT, "text/html, text/markdown".parse().unwrap());
    headers.insert(USER_AGENT, "mlc (github.com/becheran/mlc)".parse().unwrap());
    req
}

async fn http_request(url: &reqwest::Url) -> reqwest::Result<LinkCheckResult> {
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
    if status.is_success() {
        if response.url() == url {
            Ok(LinkCheckResult::Ok)
        } else {
            Ok(LinkCheckResult::Warning(
                "Request was redirected to ".to_string() + response.url().as_ref(),
            ))
        }
    } else if status.is_redirection() {
        // Only if > 10 redirects
        Ok(LinkCheckResult::Warning(status_to_string(status)))
    } else {
        debug!("Got the status code {status:?}. Retry with get-request.");
        let get_request = Request::new(Method::GET, url.clone());
        let response = CLIENT.execute(get_request).await?;
        let status = response.status();
        if status.is_success() {
            if response.url() == url {
                Ok(LinkCheckResult::Ok)
            } else {
                Ok(LinkCheckResult::Warning(status_to_string(status)))
            }
        } else {
            Ok(LinkCheckResult::Failed(status_to_string(status)))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    async fn check_http_url_str(url_str: &str) -> LinkCheckResult {
        let url = reqwest::Url::parse(url_str).expect("URL of unknown type");
        check_http(&url).await
    }

    #[tokio::test]
    async fn check_http_is_available() {
        let result = check_http_url_str("https://gitlab.com/becheran/mlc").await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_is_redirection() {
        let result = check_http_url_str("http://gitlab.com/becheran/mlc").await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(
                "Request was redirected to https://gitlab.com/becheran/mlc".to_string()
            )
        );
    }

    #[tokio::test]
    async fn check_http_is_redirection_failure() {
        let result = check_http_url_str("http://github.com/fake-page").await;
        assert_eq!(
            result,
            LinkCheckResult::Failed("404 - Not Found".to_string())
        );
    }

    #[tokio::test]
    async fn check_https_crates_io_available() {
        let result = check_http_url_str("https://crates.io").await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    // #[tokio::test]
    async fn check_http_request_with_hash() {
        let result = check_http_url_str("https://gitlab.com/becheran/mlc#bla").await;
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[tokio::test]
    async fn check_http_request_redirection_with_hash() {
        let result = check_http_url_str("http://gitlab.com/becheran/mlc#bla").await;
        assert_eq!(
            result,
            LinkCheckResult::Warning(
                "Request was redirected to https://gitlab.com/becheran/mlc".to_string()
            )
        );
    }

    #[tokio::test]
    async fn check_wrong_http_request() {
        let result = check_http_url_str("https://doesNotExist.me/even/less/likelly").await;
        assert!(result != LinkCheckResult::Ok);
    }
}
