/*
 * SPDX-FileCopyrightText: 2020 - 2024 Armin Becher <becherarmin@gmail.com>
 * SPDX-FileCopyrightText: 2025 Robin Vobruba <hoijui.quaero@gmail.com>
 *
 * SPDX-License-Identifier: MIT
 */

use crate::link_validator::LinkCheckResult;
use log::debug;
use regex::Regex;
use std::sync::LazyLock;
use url::Url;

static EMAIL_USER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^((?i)[a-z0-9_!#$%&'*+-/=?^`{|}~+.]+)$").unwrap());
static EMAIL_DOMAIN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})$").unwrap());
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
    r"^((?i)[a-z0-9_!#$%&'*+-/=?^`{|}~+]([a-z0-9_!#$%&'*+-/=?^`{|}~+.]*[a-z0-9_!#$%&'*+-/=?^_{|}~+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})$"
).unwrap()
});

pub fn check_mail(target: &Url) -> LinkCheckResult {
    debug!("Check mail target {target:#?}");
    // let mut mail = target;
    // if let Some(stripped) = target.strip_prefix("mailto://") {
    //     mail = stripped;
    // } else if let Some(stripped) = target.strip_prefix("mailto:") {
    //     mail = stripped;
    // }
    if !target.scheme().eq("mailto") {
        return LinkCheckResult::Failed(
            "Not a valid EMail address - URL-Scheme is not 'mailto'".to_string(),
        );
    }
    let url_str = target.to_string();
    let email = url_str
        .strip_prefix("mailto:")
        .expect("Missing scheme 'mailto:'");
    if !EMAIL_REGEX.is_match(email) {
        return LinkCheckResult::Failed(format!("Not a valid EMail address: '{email}'"));
    }
    LinkCheckResult::Ok
    // else if !EMAIL_USER_REGEX.is_match(target.username()) || target.username().starts_with(".") || target.username().ends_with(".") {
    //     LinkCheckResult::Failed(format!("Not a valid EMail address - User is invalid: '{}'", target.username()))
    // } else {
    //     if let Some(host) = target.host_str() {
    //         if !EMAIL_DOMAIN_REGEX.is_match(host) {
    //             LinkCheckResult::Failed(format!("Not a valid EMail address - Domain is invalid: '{host}'"))
    //         } else {
    //             LinkCheckResult::Ok
    //         }
    //     } else {
    //         LinkCheckResult::Failed("Not a valid EMail address - Domain is missing.".to_string())
    //     }
    // }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::test_case;

    fn check_email_str(email_str: &str) -> LinkCheckResult {
        reqwest::Url::parse(email_str)
            .map(|url| {
                println!("{url:#?}");
                check_mail(&url)
            })
            .unwrap_or_else(|err| {
                LinkCheckResult::Failed(format!(
                    "Failed to parse '{email_str}' as (EMail-)URL: {err}"
                ))
            })
    }

    #[test_case("mailto://+bar@bar.com")]
    #[test_case("mailto://foo+@bar.com")]
    #[test_case("mailto://foo.lastname@bar.com")]
    #[test_case("mailto://tst@xyz.us")]
    #[test_case("mailto:bla.bla@web.de")]
    #[test_case("mailto:bla.bla.ext@web.de")]
    #[test_case("mailto:BlA.bLa.ext@web.de")]
    #[test_case("mailto:foo-bar@foobar.com")]
    #[test_case("mailto:!#$%&'*+-/=?^_`{|}~-foo@foobar.com")]
    #[test_case("mailto:some@hostnumbers123.com")]
    #[test_case("mailto:some@host-name.com")]
    fn mail_links(link: &str) {
        let result = check_email_str(link);
        assert_eq!(result, LinkCheckResult::Ok);
    }

    #[test_case("mailto://@bar@bar")]
    #[test_case("mailto://foobar.com")]
    #[test_case("mailto://foo.lastname.com")]
    #[test_case("mailto:foo.do@l$astname.cOM")]
    #[test_case("mailto:foo@l_astname.cOM")]
    #[test_case("bla.bla@web.de")]
    fn invalid_mail_links(link: &str) {
        let result = check_email_str(link);
        assert!(result != LinkCheckResult::Ok);
    }
}
