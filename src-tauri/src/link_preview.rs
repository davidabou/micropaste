use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

use reqwest::redirect::Policy;
use reqwest::{Client, Response, header};
use scraper::{Html, Selector};
use serde::Serialize;
use url::{Host, Url};

use crate::images;

const MAX_HTML_BYTES: usize = 1 << 20;
const MAX_IMAGE_BYTES: usize = 5 << 20;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkPreview {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub site_name: Option<String>,
    pub image: Option<String>,
    pub icon: Option<String>,
    #[serde(skip)]
    pub fetched_at: i64,
    pub stale: bool,
}

pub const MAX_AGE_MS: i64 = 7 * 24 * 60 * 60 * 1000;

pub fn parse_url(text: &str) -> Option<Url> {
    let url = Url::parse(text.trim()).ok()?;
    (matches!(url.scheme(), "http" | "https") && is_public(&url)).then_some(url)
}

/// Never load local or internal addresses: the copied link must not reach the user's network.
fn is_public(url: &Url) -> bool {
    match url.host() {
        Some(Host::Domain(domain)) => {
            let domain = domain.to_ascii_lowercase();
            domain.contains('.')
                && ![".localhost", ".local", ".internal", ".lan", ".home"].iter().any(|suffix| domain.ends_with(suffix))
        }
        Some(Host::Ipv4(ip)) => is_public_ip(IpAddr::V4(ip)),
        Some(Host::Ipv6(ip)) => is_public_ip(IpAddr::V6(ip)),
        None => false,
    }
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            !(ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_broadcast()
                || ip.octets()[0] == 100 && (64..128).contains(&ip.octets()[1])) // Carrier-grade NAT
        }
        IpAddr::V6(ip) if let Some(v4) = ip.to_ipv4_mapped() => is_public_ip(IpAddr::V4(v4)),
        IpAddr::V6(ip) => {
            let first = ip.segments()[0];
            !(ip.is_loopback()
                || ip.is_unspecified()
                || (first & 0xfe00) == 0xfc00 // Unique local
                || (first & 0xffc0) == 0xfe80) // Link local
        }
    }
}

fn client() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(8))
            // Redirects must stay on public websites too
            .redirect(Policy::custom(|attempt| {
                if attempt.previous().len() >= 5 || !is_public(attempt.url()) {
                    attempt.stop()
                } else {
                    attempt.follow()
                }
            }))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Micropaste (link preview)")
            .build()
            .expect("valid HTTP client")
    })
}

pub async fn fetch(url: &Url, images_dir: &Path) -> reqwest::Result<LinkPreview> {
    let mut preview = LinkPreview {
        url: url.to_string(),
        title: None,
        description: None,
        site_name: None,
        image: None,
        icon: None,
        fetched_at: 0,
        stale: false,
    };

    let response = client().get(url.clone()).send().await?.error_for_status()?;
    let content_type = content_type(&response);

    if content_type.starts_with("image/") {
        preview.image = save_image(response, images_dir).await;
        return Ok(preview);
    }
    if !content_type.contains("html") {
        return Ok(preview);
    }

    let page_url = response.url().clone();
    let html = read_limited(response, MAX_HTML_BYTES).await?;
    // Html isn't Send: read it before the next await
    let (tags, icon_href) = {
        let document = Html::parse_document(&String::from_utf8_lossy(&html));
        (read_tags(&document), icon_href(&document))
    };

    let first = |keys: &[&str]| keys.iter().find_map(|key| tags.get(*key).cloned());
    preview.title = first(&["og:title", "twitter:title", "title"]);
    preview.description = first(&["og:description", "twitter:description", "description"]);
    preview.site_name = first(&["og:site_name"]);

    let image_url = first(&["og:image", "og:image:url", "og:image:secure_url", "twitter:image", "twitter:image:src"])
        .and_then(|image| page_url.join(&image).ok())
        .filter(is_public);
    if let Some(image_url) = image_url
        && let Ok(response) = client().get(image_url).send().await.and_then(|r| r.error_for_status())
    {
        preview.image = save_image(response, images_dir).await;
    }

    let icon_url = icon_href
        .and_then(|href| page_url.join(&href).ok())
        .or_else(|| page_url.join("/favicon.ico").ok())
        .filter(is_public);
    if let Some(icon_url) = icon_url
        && let Ok(response) = client().get(icon_url).send().await.and_then(|r| r.error_for_status())
    {
        preview.icon = save_image(response, images_dir).await;
    }

    Ok(preview)
}

/// The sharpest icon the page declares, the touch icon only when it declares no other
fn icon_href(document: &Html) -> Option<String> {
    let link = Selector::parse("link[rel][href]").unwrap();
    document
        .select(&link)
        .filter_map(|element| {
            let element = element.value();
            let href = element.attr("href")?.trim();
            let rel = element.attr("rel")?.to_ascii_lowercase();
            let touch = rel.split_whitespace().any(|token| token.starts_with("apple-touch-icon"));
            if href.is_empty() || !(touch || rel.split_whitespace().any(|token| token == "icon")) {
                return None;
            }
            let svg = element.attr("type").is_some_and(|kind| kind.contains("svg"))
                || href.split(['?', '#']).next().is_some_and(|path| path.to_ascii_lowercase().ends_with(".svg"));
            let size = element
                .attr("sizes")
                .unwrap_or_default()
                .split_whitespace()
                .filter_map(|size| {
                    if size.eq_ignore_ascii_case("any") {
                        return Some(u32::MAX);
                    }
                    size.to_ascii_lowercase().split_once('x')?.0.parse().ok()
                })
                .max()
                .unwrap_or(if svg { u32::MAX } else { 0 });
            Some(((!touch, size), href.to_owned()))
        })
        .max_by_key(|(rank, _)| *rank)
        .map(|(_, href)| href)
}

fn read_tags(document: &Html) -> HashMap<String, String> {
    let meta = Selector::parse("meta[content]").unwrap();
    let title = Selector::parse("title").unwrap();

    let mut tags = HashMap::new();
    for element in document.select(&meta) {
        let element = element.value();
        let Some(key) = element.attr("property").or_else(|| element.attr("name")) else {
            continue;
        };
        let value = element.attr("content").unwrap_or_default().trim();
        if !value.is_empty() {
            tags.entry(key.to_ascii_lowercase()).or_insert_with(|| value.to_owned());
        }
    }
    if let Some(element) = document.select(&title).next() {
        let text = element.text().collect::<String>();
        let text = text.trim();
        if !text.is_empty() {
            tags.entry("title".to_owned()).or_insert_with(|| text.to_owned());
        }
    }
    tags
}

async fn save_image(response: Response, images_dir: &Path) -> Option<String> {
    let extension = match content_type(&response).as_str() {
        "image/jpeg" | "image/jpg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/avif" => "avif",
        "image/svg+xml" => "svg",
        "image/x-icon" | "image/vnd.microsoft.icon" => "ico",
        "image/bmp" => "bmp",
        _ => return None,
    };
    let bytes = read_limited(response, MAX_IMAGE_BYTES).await.ok()?;
    if bytes.len() >= MAX_IMAGE_BYTES {
        return None;
    }

    let file = format!("{}.{extension}", blake3::hash(&bytes).to_hex());
    images::save_file(images_dir, &file, &bytes).ok()?;
    Some(file)
}

fn content_type(response: &Response) -> String {
    response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
}

async fn read_limited(mut response: Response, limit: usize) -> reqwest::Result<Vec<u8>> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        body.extend_from_slice(&chunk);
        if body.len() >= limit {
            body.truncate(limit);
            break;
        }
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_public_web_links() {
        for url in [
            "https://example.com",
            " http://www.example.co.uk/page?q=1 \n",
            "https://93.184.216.34/",
            "https://[2606:4700::1111]/",
        ] {
            assert!(parse_url(url).is_some(), "{url}");
        }
    }

    #[test]
    fn rejects_other_schemes_and_plain_text() {
        for text in ["ftp://example.com", "file:///C:/Windows", "javascript:alert(1)", "example.com", "hello world", ""]
        {
            assert!(parse_url(text).is_none(), "{text}");
        }
    }

    #[test]
    fn rejects_local_and_internal_hosts() {
        for url in [
            "http://localhost:3000",
            "http://intranet/",
            "http://app.localhost",
            "http://printer.local",
            "http://nas.LAN",
            "http://router.home",
            "http://api.internal",
            "http://127.0.0.1",
            "http://127.1",
            "http://2130706433",
            "http://0.0.0.0",
            "http://10.0.0.1",
            "http://172.16.0.1",
            "http://192.168.1.1",
            "http://169.254.169.254/latest/meta-data",
            "http://100.64.0.1",
            "http://255.255.255.255",
            "http://[::1]",
            "http://[::]",
            "http://[fd00::1]",
            "http://[fe80::1]",
            "http://[::ffff:127.0.0.1]",
            "http://[::ffff:192.168.1.1]",
        ] {
            assert!(parse_url(url).is_none(), "{url}");
        }
    }

    #[test]
    fn reads_open_graph_tags_first_and_falls_back_to_the_title() {
        let tags = read_tags(&Html::parse_document(
            r#"<html><head>
                <title> Page title </title>
                <meta property="og:title" content="OG title">
                <meta property="og:title" content="Second OG title">
                <meta name="Description" content="  A description  ">
                <meta name="twitter:image" content="">
                <meta charset="utf-8">
            </head></html>"#,
        ));
        assert_eq!(tags.get("og:title").map(String::as_str), Some("OG title"));
        assert_eq!(tags.get("title").map(String::as_str), Some("Page title"));
        assert_eq!(tags.get("description").map(String::as_str), Some("A description"));
        assert!(!tags.contains_key("twitter:image"));
    }

    #[test]
    fn reads_broken_html() {
        let tags = read_tags(&Html::parse_document("<title>Unclosed <meta name=description content=x"));
        assert!(tags.contains_key("title"));
        assert!(read_tags(&Html::parse_document("")).is_empty());
    }

    #[test]
    fn picks_the_sharpest_icon() {
        let icon = |html: &str| icon_href(&Html::parse_document(html));
        assert_eq!(
            icon(
                r#"<link rel="apple-touch-icon" sizes="180x180" href="/touch.png">
                <link rel="shortcut icon" href="/favicon.ico">
                <link rel="icon" sizes="16x16 32x32" href="/32.png">
                <link rel="stylesheet" href="/style.css">"#
            )
            .as_deref(),
            Some("/32.png")
        );
        assert_eq!(
            icon(r#"<link rel="icon" sizes="192x192" href="/192.png"><link rel="icon" href="/icon.svg">"#).as_deref(),
            Some("/icon.svg")
        );
        assert_eq!(icon(r#"<link rel="apple-touch-icon" href="/touch.png">"#).as_deref(), Some("/touch.png"));
        assert_eq!(icon(r#"<link rel="icon" href=" ">"#), None);
    }
}
