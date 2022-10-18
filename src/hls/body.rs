use lazy_static::lazy_static;
use regex::{Captures, Regex};

lazy_static! {
    static ref RE_URL: Regex = Regex::new(r"https://(.*?)/").unwrap();
    static ref RE_PATH: Regex = Regex::new("(/(?:api|videoplayback)(?:.[^\"\n]+))").unwrap();
    static ref RE_XMAP: Regex = Regex::new("(#EXT-X-MAP:URI=\".*?)(?:?)(host=.*\")").unwrap();
}

pub async fn modify_hls_body(body: &str, host_url: &str) -> Result<String, ()> {
    let body_relative_urls = RE_URL.replace_all(body, "/").to_string();
    let body_add_host = RE_PATH
        .replace_all(&body_relative_urls, |caps: &Captures| {
            format!("{}?host={}", &caps[1], host_url)
        })
        .to_string();

    if body_add_host.contains("#EXT-X-MAP:URI=\"") {
        let body_fix_query = RE_XMAP
            .replace_all(&body_add_host, |caps: &Captures| {
                format!("{}&{}", &caps[1], &caps[2])
            })
            .to_string();
        return Ok(body_fix_query.clone());
    }
    Ok(body_add_host.clone())
}
