//! Literal version matching for publisher download-page anchor links.

use std::collections::BTreeSet;
use std::time::Duration;

use crate::context::VersionInfo;
use crate::error::{Error, Result};

use super::StarlarkProvider;

impl StarlarkProvider {
    pub(super) async fn resolve_fetch_html_versions_descriptor(
        &self,
        descriptor: &serde_json::Value,
    ) -> Result<Vec<VersionInfo>> {
        let required = |field: &str| {
            descriptor
                .get(field)
                .and_then(|value| value.as_str())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    Error::EvalError(format!("fetch_html_versions requires nonempty '{field}'"))
                })
        };
        let url = required("url")?;
        let href_prefix = required("href_prefix")?;
        let filename_prefix = required("filename_prefix")?;
        let filename_suffix = required("filename_suffix")?;
        for (field, value) in [("url", url), ("href_prefix", href_prefix)] {
            let parsed = reqwest::Url::parse(value).map_err(|error| {
                Error::EvalError(format!("fetch_html_versions invalid {field}: {error}"))
            })?;
            if !matches!(parsed.scheme(), "http" | "https")
                || parsed.host_str().is_none()
                || !parsed.username().is_empty()
                || parsed.password().is_some()
                || parsed.query().is_some()
                || parsed.fragment().is_some()
            {
                return Err(Error::EvalError(format!(
                    "fetch_html_versions invalid {field}"
                )));
            }
        }
        if !href_prefix.ends_with('/') {
            return Err(Error::EvalError(
                "fetch_html_versions href_prefix must end with '/'".into(),
            ));
        }
        if [filename_prefix, filename_suffix]
            .iter()
            .any(|value| value.contains(['/', '?', '#']))
        {
            return Err(Error::EvalError(
                "fetch_html_versions filename_prefix and filename_suffix must be filename literals"
                    .into(),
            ));
        }
        if !matches!(
            descriptor.get("version_filter"),
            None | Some(serde_json::Value::Null)
        ) && descriptor
            .get("version_filter")
            .and_then(|value| value.as_str())
            != Some("numeric")
        {
            return Err(Error::EvalError(
                "fetch_html_versions version_filter must be 'numeric'".into(),
            ));
        }
        let excluded: Vec<&str> = match descriptor.get("exclude_version_suffixes") {
            None => Vec::new(),
            Some(serde_json::Value::Array(values)) => values.iter().map(|value| {
                value.as_str().filter(|value| !value.is_empty()).ok_or_else(|| Error::EvalError(
                    "fetch_html_versions exclude_version_suffixes must contain nonempty strings".into()))
            }).collect::<Result<_>>()?,
            Some(_) => return Err(Error::EvalError("fetch_html_versions exclude_version_suffixes must be an array".into())),
        };

        let client = reqwest::Client::builder()
            .user_agent("vx (https://github.com/loonghao/vx)")
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|error| Error::EvalError(format!("HTML version client: {error}")))?;
        let response = client.get(url).send().await.map_err(|error| {
            Error::EvalError(format!("HTML version fetch failed for {url}: {error}"))
        })?;
        if !response.status().is_success() {
            return Err(Error::EvalError(format!(
                "HTML version fetch failed for {url}: HTTP {}",
                response.status()
            )));
        }
        let html = response.text().await.map_err(|error| {
            Error::EvalError(format!("HTML version body failed for {url}: {error}"))
        })?;

        let mut found = BTreeSet::new();
        for href in anchor_hrefs(&html) {
            let Some(path) = href.strip_prefix(href_prefix) else {
                continue;
            };
            // Match only literal stable asset paths, never query strings or traversal.
            if path.contains(['?', '#', '%', '&', '\\'])
                || path.split('/').any(|part| matches!(part, "." | ".."))
            {
                continue;
            }
            let Some(version) = path
                .rsplit('/')
                .next()
                .and_then(|name| name.strip_prefix(filename_prefix))
                .and_then(|name| name.strip_suffix(filename_suffix))
            else {
                continue;
            };
            let components: Vec<_> = version.split('.').collect();
            if components.len() >= 2
                && components
                    .iter()
                    .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
                && !excluded.iter().any(|suffix| version.ends_with(suffix))
            {
                found.insert(version.to_owned());
            }
        }
        if found.is_empty() {
            return Err(Error::EvalError(format!(
                "fetch_html_versions: no matching published assets at {url}"
            )));
        }
        let mut versions: Vec<_> = found
            .into_iter()
            .map(|version| VersionInfo {
                version,
                lts: false,
                stable: true,
                date: None,
            })
            .collect();
        versions.sort_by(|a, b| numeric_version_cmp(&b.version, &a.version));
        Ok(versions)
    }
}

fn numeric_version_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    let mut left = a.split('.');
    let mut right = b.split('.');
    loop {
        match (left.next(), right.next()) {
            (Some(a), Some(b)) => {
                let a = a.trim_start_matches('0');
                let b = b.trim_start_matches('0');
                let order = a.len().cmp(&b.len()).then_with(|| a.cmp(b));
                if !order.is_eq() {
                    return order;
                }
            }
            (a, b) => return a.is_some().cmp(&b.is_some()),
        }
    }
}

/// A small tokenizer for literal anchor attributes, not a general HTML parser.
/// Entities and encoded asset paths are deliberately not interpreted.
fn anchor_hrefs(html: &str) -> Vec<&str> {
    let bytes = html.as_bytes();
    let lower = html.to_ascii_lowercase();
    let mut offset = 0;
    let mut hrefs = Vec::new();
    while let Some(relative) = html[offset..].find('<') {
        let start = offset + relative;
        if html[start..].starts_with("<!--") {
            let Some(end) = html[start + 4..].find("-->") else {
                break;
            };
            offset = start + 4 + end + 3;
            continue;
        }
        let name_start = start + 1;
        let mut name_end = name_start;
        while name_end < bytes.len()
            && !is_html_whitespace(&bytes[name_end])
            && !matches!(bytes[name_end], b'/' | b'>')
        {
            name_end += 1;
        }
        let Some(end) = tag_end(bytes, name_end) else {
            break;
        };
        offset = end + 1;
        let name = &lower[name_start..name_end];
        if name == "plaintext" {
            break;
        }
        if matches!(
            name,
            "script" | "style" | "textarea" | "title" | "iframe" | "xmp" | "noembed" | "noframes"
        ) {
            let closing = format!("</{name}");
            loop {
                let Some(relative) = lower[offset..].find(&closing) else {
                    return hrefs;
                };
                let after_name = offset + relative + closing.len();
                if bytes
                    .get(after_name)
                    .is_some_and(|byte| is_html_whitespace(byte) || matches!(byte, b'>' | b'/'))
                {
                    offset = tag_end(bytes, after_name).map_or(bytes.len(), |end| end + 1);
                    break;
                }
                offset = after_name;
            }
        } else if name == "a"
            && let Some(href) = anchor_href(&html[name_end..end])
        {
            hrefs.push(href);
        }
    }
    hrefs
}

fn is_html_whitespace(byte: &u8) -> bool {
    matches!(byte, b'\t' | b'\n' | b'\x0c' | b'\r' | b' ')
}

fn tag_end(bytes: &[u8], mut offset: usize) -> Option<usize> {
    let mut quote = None;
    while let Some(&byte) = bytes.get(offset) {
        match (quote, byte) {
            (Some(open), close) if open == close => quote = None,
            (None, b'\'' | b'"') => quote = Some(byte),
            (None, b'>') => return Some(offset),
            _ => {}
        }
        offset += 1;
    }
    None
}

fn anchor_href(attributes: &str) -> Option<&str> {
    let bytes = attributes.as_bytes();
    let mut offset = 0;
    while offset < bytes.len() {
        while bytes
            .get(offset)
            .is_some_and(|byte| is_html_whitespace(byte) || *byte == b'/')
        {
            offset += 1;
        }
        let start = offset;
        while bytes
            .get(offset)
            .is_some_and(|byte| !is_html_whitespace(byte) && !matches!(byte, b'=' | b'/'))
        {
            offset += 1;
        }
        let name = &attributes[start..offset];
        while bytes.get(offset).is_some_and(is_html_whitespace) {
            offset += 1;
        }
        let mut value = "";
        if bytes.get(offset) == Some(&b'=') {
            offset += 1;
            while bytes.get(offset).is_some_and(is_html_whitespace) {
                offset += 1;
            }
            let quote = bytes
                .get(offset)
                .copied()
                .filter(|byte| matches!(byte, b'\'' | b'"'));
            offset += usize::from(quote.is_some());
            let start = offset;
            while bytes.get(offset).is_some_and(|byte| match quote {
                Some(quote) => *byte != quote,
                None => !is_html_whitespace(byte),
            }) {
                offset += 1;
            }
            value = &attributes[start..offset];
            offset += usize::from(quote.is_some());
        }
        if name.eq_ignore_ascii_case("href") {
            return Some(value);
        }
        // Malformed attributes must still advance the tokenizer.
        if offset == start {
            offset += 1;
        }
    }
    None
}
