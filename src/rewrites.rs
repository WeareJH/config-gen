use regex::Regex;
use regex::Captures;
use std::borrow::Cow;
use url::Url;

///
/// Replace the host name in a string
///
/// # Examples
///
/// ```rust
/// let bytes = "<a href=\"https://www.acme.com\">Home</a>";
/// let expected = "<a href=\"https://127.0.0.1:8000\">Home</a>";
/// assert_eq!(expected, replace_host(bytes, "www.acme.com", "127.0.0.1:8000"));
/// ```
///
pub fn replace_host<'a>(bytes: &'a str, host_to_replace: &'a str, target_host: &'a str, target_port: u16) -> Cow<'a, str> {
    let matcher = format!("https?:(?:\\\\)?/(?:\\\\)?/{}", host_to_replace);
    Regex::new(&matcher)
        .unwrap()
        .replace_all(bytes,
                     |item: &Captures|
                         modify_url(item, target_host, target_port).unwrap_or(String::from("")))
}

pub fn replace_cookie_domain_on_page<'a>(bytes: &'a str, host_to_replace: &str) -> Cow<'a, str> {
    let matcher = format!(r#""domain": ".{}","#, host_to_replace);
    Regex::new(&matcher)
        .unwrap()
        .replace_all(bytes, "")
}

#[test]
fn test_replace_cookie_domain_on_page() {
    let bytes = r#"
        <script type="text/x-magento-init">
            {
                "*": {
                    "mage/cookies": {
                        "expires": null,
                        "path": "/",
                        "domain": ".www.neomorganics.com",
                        "secure": false,
                        "lifetime": "10800"
                    }
                }
            }
        </script>
    "#;
    let replaced = replace_cookie_domain_on_page(&bytes, "www.neomorganics.com");
    println!("-> {}", replaced);
}

// Attempt to modify the matched URL,
// note: this can fail at multiple points
// and if it does we just want a None and we move on
// there's no benefit to handling the error in any case here
fn modify_url(caps: &Captures, host: &str, port: u16) -> Option<String> {
    let first_match = caps.iter().nth(0)?;
    let match_item = first_match?;
    let mut url = Url::parse(match_item.as_str()).ok()?;

    url.set_host(Some(host)).ok()?;
    url.set_port(Some(port)).ok()?;
    let mut as_string = url.to_string();
    as_string.pop();
    Some(as_string)
}

#[test]
fn test_rewrites() {
    let bytes = "
    <a href=\"https://www.acme.com\">Home</a>
    <a href=\"http://www.acme.com\">Home</a>
    ";
    let expected = "
    <a href=\"https://127.0.0.1:8080\">Home</a>
    <a href=\"http://127.0.0.1:8080\">Home</a>
    ";
    let actual = replace_host(bytes, "www.acme.com", "127.0.0.1", 8080);
    assert_eq!(actual, expected);
}
#[test]
fn test_rewrites_within_escaped_json() {
    let bytes = r#"
    {"url": "https:\/\/www.acme.com\/checkout\/cart\/\"}
    "#;
    let expected = r#"
    {"url": "https://127.0.0.1:8080\/checkout\/cart\/\"}
    "#;
    let actual = replace_host(bytes, "www.acme.com", "127.0.0.1", 8080);
    println!("actual={}", actual);
    assert_eq!(actual, expected);
}
