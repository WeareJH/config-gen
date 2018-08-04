use regex::Regex;
use regex::Captures;
use std::borrow::Cow;
use url::{Url};

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
    let matcher = format!("https?://{}", host_to_replace);
    Regex::new(&matcher).unwrap().replace_all(bytes, |item: &Captures| main_replace(item, target_host, target_port))
}

fn main_replace(caps: &Captures, host: &str, port: u16) -> String {
    caps.iter().nth(0)
        .map_or(String::from(""),
            // here there was a regex match
            |capture_item| capture_item.map_or(String::from(""),
                   // here we have access to the individual match group
                   |item| {
                       // now we can try to parse the url
                       match Url::parse(item.as_str()) {
                           // if it parsed, we set the url to the value passed in
                           Ok(mut url) => {
                               match url.set_host(Some(host)) {
                                   Ok(()) => match url.set_port(Some(port)) {
                                       Ok(_) => {
                                           println!("setting: {}", url.to_string());
                                           url.to_string()
                                       },
                                       Err(_) => {
                                           eprintln!("Could not set port");
                                           String::from(item.as_str())
                                       }
                                   },
                                   Err(_) => {
                                       eprintln!("Could not set host");
                                       String::from(item.as_str())
                                   }
                               }
                           }
                           Err(_) => {
                               eprintln!("Could not parse url");
                               String::from(item.as_str())
                           }
                       }
                   }))
}

#[test]
fn test_rewrites() {
    let bytes = "
    <a href=\"https://www.neomorganics.com\">Home</a>
    <a href=\"http://www.neomorganics.com\">Home</a>
    ";
    let expected = "
    <a href=\"https://127.0.0.1:8080/\">Home</a>
    <a href=\"http://127.0.0.1:8080/\">Home</a>
    ";
    let matcher = format!("https?://{}", "https://www.neomorganics.com");
    let actual = Regex::new(&matcher).unwrap().replace_all(bytes, |item: &Captures| main_replace(item, "127.0.0.1", 8080));
}
