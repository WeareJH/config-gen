use rewrites::RewriteContext;
use regex::Regex;

///
/// # Examples
///
///
pub trait Replacer {
    type Item;
    type Options;
    type Output;

    fn apply(self, opts: &Self::Options, items: Vec<Self::Item>) -> Self::Output;
}

pub struct Subject(String);

impl Subject {
    pub fn new(input: impl Into<String>) -> Subject {
        Subject(input.into())
    }
}

impl Replacer for Subject {
    type Item = fn(&str, &Self::Options) -> String;
    type Options = RewriteContext;
    type Output = String;

    fn apply(self, opts: &Self::Options, items: Vec<Self::Item>) -> Self::Output {
        items.iter().fold(self.0, |acc, item_fn| item_fn(&acc, &opts))
    }
}


#[test]
fn test_subject_replacer() {
    let s = Subject::new(r#"<a href="https://acme.m2/path">Click</a>"#);
    let ctx = RewriteContext{
        host_to_replace: String::from("acme.m2"),
        target_host: String::from("127.0.0.1"),
        target_port: 8080,
    };
    fn replacer(input: &str, opts: &RewriteContext) -> String {
        Regex::new(&opts.host_to_replace)
            .unwrap()
            .replace_all(input, opts.target_host.as_str())
            .to_string()
    }
    let updated = s.apply(&ctx, vec![
        replacer
    ]);
    println!("{}", updated);
}