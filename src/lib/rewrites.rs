use regex::Captures;
use regex::Regex;
use url::Url;

///
/// A struct containing values that may be of
/// interest to a replacer function
///
/// # Examples
///
/// ```rust
/// use bs::rewrites::*;
///
/// let opts = RewriteContext::new("www.acme.com")
///     .with_target("127.0.0.1", 8000);
///
/// assert_eq!(opts.host_to_replace, String::from("www.acme.com"))
/// ```
///
#[derive(Default, Serialize)]
pub struct RewriteContext {
    pub host_to_replace: String,
    pub target_host: String,
    pub target_port: u16,
}

impl RewriteContext {
    pub fn new(host: impl Into<String>) -> RewriteContext {
        RewriteContext {
            host_to_replace: host.into(),
            ..Default::default()
        }
    }
    pub fn with_target(mut self, host: impl Into<String>, port: u16) -> RewriteContext {
        self.target_host = host.into();
        self.target_port = port;
        self
    }
}

///
/// Replace the host name in a string
///
/// # Examples
///
/// ```rust
/// use bs::rewrites::*;
///
/// let bytes = "<a href=\"https://www.acme.com\">Home</a>";
/// let expected = "<a href=\"https://127.0.0.1:8000\">Home</a>";
///
/// let opts = RewriteContext::new("www.acme.com")
///     .with_target("127.0.0.1", 8000);
///
/// assert_eq!(expected, replace_host(bytes, &opts));
/// ```
///
pub fn replace_host(bytes: &str, context: &RewriteContext) -> String {
    let matcher = format!("https?:(?:\\\\)?/(?:\\\\)?/{}", context.host_to_replace);
    Regex::new(&matcher)
        .unwrap()
        .replace_all(bytes, |item: &Captures| {
            modify_url(item, &context).unwrap_or(String::from(""))
        }).to_string()
}

///
/// Attempt to modify a URL,
///
/// note: this can fail at multiple points
/// and if it does we just want a None and we move on
/// there's no benefit to handling the error in any case here
///
pub fn modify_url(caps: &Captures, context: &RewriteContext) -> Option<String> {
    let first_match = caps.iter().nth(0)?;
    let match_item = first_match?;
    let mut url = Url::parse(match_item.as_str()).ok()?;

    url.set_host(Some(&context.target_host)).ok()?;
    url.set_port(Some(context.target_port)).ok()?;
    let mut as_string = url.to_string();
    as_string.pop();
    Some(as_string)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let context = RewriteContext {
            host_to_replace: String::from("www.acme.com"),
            target_host: String::from("127.0.0.1"),
            target_port: 8080,
        };
        let actual = replace_host(bytes, &context);
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
        let context = RewriteContext {
            host_to_replace: String::from("www.acme.com"),
            target_host: String::from("127.0.0.1"),
            target_port: 8080,
        };
        let actual = replace_host(bytes, &context);
        println!("actual={}", actual);
        assert_eq!(actual, expected);
    }
}
