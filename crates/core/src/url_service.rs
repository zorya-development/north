use regex::Regex;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{debug, warn};

static BARE_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    // Match URLs — filtering against existing markdown links is done at the call site
    Regex::new(r"https?://[^\s)<>\[\]]+").unwrap()
});

static MD_LINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[([^\]]*)\]\([^)]+\)").unwrap());

/// Fast sync check: returns true if text contains bare URLs not already in markdown links.
pub fn has_bare_urls(text: &str) -> bool {
    let md_link_ranges: Vec<(usize, usize)> = MD_LINK_RE
        .find_iter(text)
        .map(|m| (m.start(), m.end()))
        .collect();

    BARE_URL_RE.find_iter(text).any(|m| {
        !md_link_ranges
            .iter()
            .any(|&(start, end)| m.start() >= start && m.end() <= end)
    })
}

/// Resolve bare URLs in text to `[Page Title](url)` markdown links.
/// URLs already in `[text](url)` format are left unchanged.
/// Failures (timeout, network error, no title) leave the bare URL as-is.
pub async fn resolve_urls_in_text(text: &str) -> String {
    // Find all bare URLs that aren't part of existing markdown links
    let md_link_ranges: Vec<(usize, usize)> = MD_LINK_RE
        .find_iter(text)
        .map(|m| (m.start(), m.end()))
        .collect();

    let bare_urls: Vec<(usize, usize, &str)> = BARE_URL_RE
        .find_iter(text)
        .filter(|m| {
            !md_link_ranges
                .iter()
                .any(|&(start, end)| m.start() >= start && m.end() <= end)
        })
        .map(|m| (m.start(), m.end(), m.as_str()))
        .collect();

    if bare_urls.is_empty() {
        debug!(text, "url_service: no bare URLs found in text");
        return text.to_string();
    }

    debug!(count = bare_urls.len(), "url_service: found bare URLs");
    for (_, _, url) in &bare_urls {
        debug!(url, "url_service: detected bare URL");
    }

    let mut result = text.to_string();
    // Process in reverse order to preserve offsets
    for (start, end, url) in bare_urls.into_iter().rev() {
        debug!(url, "url_service: fetching page title");
        match fetch_page_title(url).await {
            Some(title) => {
                debug!(url, title = title.as_str(), "url_service: resolved title");
                let sanitized_title = title.replace('[', "\\[").replace(']', "\\]");
                let md_link = format!("[{}]({})", sanitized_title, url);
                result.replace_range(start..end, &md_link);
            }
            None => {
                warn!(
                    url,
                    "url_service: failed to resolve title, keeping bare URL"
                );
            }
        }
    }

    result
}

async fn fetch_page_title(url: &str) -> Option<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .redirect(reqwest::redirect::Policy::limited(5))
        .build()
        .ok()?;

    let resp = match client
        .get(url)
        .header("User-Agent", "North-GTD/1.0")
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            warn!(url, error = %e, "url_service: HTTP request failed");
            return None;
        }
    };

    let status = resp.status();
    if !status.is_success() {
        warn!(url, %status, "url_service: non-success status");
        return None;
    }

    // Only process HTML responses
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !content_type.contains("text/html") {
        warn!(url, content_type, "url_service: not HTML, skipping");
        return None;
    }

    debug!(url, content_type, "url_service: got HTML response");

    // Limit body to 1MB (sites like YouTube put title tags 600KB+ into the HTML)
    let body = match resp.text().await {
        Ok(b) => b,
        Err(e) => {
            warn!(url, error = %e, "url_service: failed to read response body");
            return None;
        }
    };

    let truncated = body.len() > 1_048_576;
    let body = if truncated {
        debug!(url, len = body.len(), "url_service: truncating body to 1MB");
        &body[..1_048_576]
    } else {
        &body
    };

    let document = scraper::Html::parse_document(body);

    // Try <title> first, then fall back to <meta property="og:title"> / <meta name="title">
    let title = extract_title(&document)
        .or_else(|| extract_meta_content(&document, "og:title"))
        .or_else(|| extract_meta_name_content(&document, "title"));

    match title {
        Some(ref t) if !t.trim().is_empty() => {
            let trimmed = t.trim().to_string();
            debug!(
                url,
                title = trimmed.as_str(),
                "url_service: extracted title"
            );
            Some(trimmed)
        }
        _ => {
            warn!(url, truncated, "url_service: no title found in HTML");
            None
        }
    }
}

fn extract_title(document: &scraper::Html) -> Option<String> {
    let selector = scraper::Selector::parse("title").ok()?;
    document
        .select(&selector)
        .next()
        .map(|el| el.text().collect::<String>())
}

fn extract_meta_content(document: &scraper::Html, property: &str) -> Option<String> {
    let selector = scraper::Selector::parse(&format!("meta[property=\"{property}\"]")).ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content").map(String::from))
}

fn extract_meta_name_content(document: &scraper::Html, name: &str) -> Option<String> {
    let selector = scraper::Selector::parse(&format!("meta[name=\"{name}\"]")).ok()?;
    document
        .select(&selector)
        .next()
        .and_then(|el| el.value().attr("content").map(String::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_urls() {
        let result = resolve_urls_in_text("Hello world").await;
        assert_eq!(result, "Hello world");
    }

    #[tokio::test]
    async fn test_existing_markdown_link_preserved() {
        let input = "Check [Google](https://google.com) for info";
        let result = resolve_urls_in_text(input).await;
        assert_eq!(result, input);
    }

    #[test]
    fn test_has_bare_urls_with_bare_url() {
        assert!(has_bare_urls("Check https://example.com for info"));
    }

    #[test]
    fn test_has_bare_urls_no_urls() {
        assert!(!has_bare_urls("Hello world"));
    }

    #[test]
    fn test_has_bare_urls_markdown_link_only() {
        assert!(!has_bare_urls(
            "Check [Google](https://google.com) for info"
        ));
    }

    #[test]
    fn test_has_bare_urls_mixed() {
        assert!(has_bare_urls(
            "[Google](https://google.com) and https://example.com"
        ));
    }
}
