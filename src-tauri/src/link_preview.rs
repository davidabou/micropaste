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

/// Only the `<head>` matters, which is near the start of the page
const MAX_HTML_BYTES: usize = 1 << 20;
const MAX_IMAGE_BYTES: usize = 5 << 20;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkPreview {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub site_name: Option<String>,
    /// File name of the preview image in the images folder
    pub image: Option<String>,
    /// Unix timestamp in milliseconds of the last load from the website
    #[serde(skip)]
    pub fetched_at: i64,
    /// Older than `MAX_AGE_MS`: shown as is, but worth reloading
    pub stale: bool,
}

/// Previews older than this are reloaded in the background when shown again.
pub const MAX_AGE_MS: i64 = 7 * 24 * 60 * 60 * 1000;

/// The URL if the copied text is a single http(s) link to a public website.
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

/// Loads the page and reads its title, description and image (Open Graph tags,
/// the ones social networks use for their link previews).
/// Errors are network failures: worth retrying later, unlike a page without preview tags.
pub async fn fetch(url: &Url, images_dir: &Path) -> reqwest::Result<LinkPreview> {
    let mut preview = LinkPreview {
        url: url.to_string(),
        title: None,
        description: None,
        site_name: None,
        image: None,
        fetched_at: 0,
        stale: false,
    };

    let response = client().get(url.clone()).send().await?.error_for_status()?;
    let content_type = content_type(&response);

    // A direct link to an image: the image is the preview
    if content_type.starts_with("image/") {
        preview.image = save_image(response, images_dir).await;
        return Ok(preview);
    }
    if !content_type.contains("html") {
        return Ok(preview);
    }

    let page_url = response.url().clone();
    let html = read_limited(response, MAX_HTML_BYTES).await?;
    let tags = read_tags(&String::from_utf8_lossy(&html));

    let first = |keys: &[&str]| keys.iter().find_map(|key| tags.get(*key).cloned());
    preview.title = first(&["og:title", "twitter:title", "title"]);
    preview.description = first(&["og:description", "twitter:description", "description"]);
    preview.site_name = first(&["og:site_name"]);

    let image_url = first(&["og:image", "og:image:url", "og:image:secure_url", "twitter:image", "twitter:image:src"])
        .and_then(|image| page_url.join(&image).ok())
        .filter(is_public);
    if let Some(image_url) = image_url {
        // A broken image doesn't prevent the rest of the preview
        if let Ok(response) = client().get(image_url).send().await.and_then(|r| r.error_for_status()) {
            preview.image = save_image(response, images_dir).await;
        }
    }

    Ok(preview)
}

/// Meta tags by name (`og:title`, `description`…) plus the page `<title>`, first occurrence wins.
fn read_tags(html: &str) -> HashMap<String, String> {
    let document = Html::parse_document(html);
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

/// Downloads the image into the images folder, named after its content hash.
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
    // Truncated: the image is too large to keep
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

/// Reads the body up to `limit` bytes, without downloading the rest.
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
