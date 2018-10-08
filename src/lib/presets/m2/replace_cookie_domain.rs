use regex::Regex;
use rewrites::RewriteContext;

///
/// Remove an on-page cookie domain (usually in JSON blobs with Magento)
///
pub fn rewrite(bytes: &str, context: &RewriteContext) -> String {
    let matcher = format!(r#""domain": ".{}","#, context.host_to_replace);
    Regex::new(&matcher)
        .unwrap()
        .replace_all(bytes, "")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_cookie_domain_on_page() {
        let bytes = r#"
        <script type="text/x-magento-init">
            {
                "*": {
                    "mage/cookies": {
                        "expires": null,
                        "path": "/",
                        "domain": ".www.acme.com",
                        "secure": false,
                        "lifetime": "10800"
                    }
                }
            }
        </script>
    "#;
        let replaced = rewrite(
            &bytes,
            &RewriteContext {
                host_to_replace: String::from("www.acme.com"),
                target_host: String::from("127.0.0.1"),
                target_port: 80,
            },
        );
        println!("-> {}", replaced);
    }
}
