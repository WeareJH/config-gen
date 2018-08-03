use regex::Regex;

#[test]
fn test_rewrites() {
    let re = Regex::new("neom.com").unwrap();
    let bytes = "
    <a href=\"https://neom.com\">Home</a>
    ";
    let expected = "
    <a href=\"https://127.0.0.1\">Home</a>
    ";
    assert_eq!(re.replace_all(bytes, "127.0.0.1"), expected);
}