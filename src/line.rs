pub mod error;

use std::path::PathBuf;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref SLS_SPEC_RE: Regex =
        Regex::new(r#"^\s*(?<target>[^\s"]+|"[^"]+")\s+(?<link>[^\s"]+|"[^"]+")\s*$"#).unwrap();
}

#[derive(Debug)]
pub enum LineType {
    Invalid,
    Empty,
    Comment,
    SlsSpec { target: PathBuf, link: PathBuf },
}

pub fn line_type(line: &str) -> LineType {
    if line.starts_with("//") {
        LineType::Comment
    } else if line.is_empty() {
        LineType::Empty
    } else {
        match SLS_SPEC_RE.captures(line) {
            Some(caps) => {
                let mut target = PathBuf::new();
                target.push(&caps["target"]);
                let mut link = PathBuf::new();
                link.push(&caps["link"]);
                LineType::SlsSpec { target, link }
            }
            None => LineType::Invalid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SLS_SPEC_RE;

    #[derive(Debug)]
    struct TestCase {
        input: String,
        matches: bool,
        target: Option<String>,
        link: Option<String>,
    }

    #[test]
    fn sls_spec_re_matches_when_it_should() {
        let test_cases = vec![
            // regular input
            TestCase {
                input: String::from("/some/random/target /some/random/link"),
                matches: true,
                target: Some(String::from("/some/random/target")),
                link: Some(String::from("/some/random/link")),
            },
            // spaces before
            TestCase {
                input: String::from("     /some/random/target /some/random/link"),
                matches: true,
                target: Some(String::from("/some/random/target")),
                link: Some(String::from("/some/random/link")),
            },
            // spaces in between
            TestCase {
                input: String::from("/some/random/target     /some/random/link"),
                matches: true,
                target: Some(String::from("/some/random/target")),
                link: Some(String::from("/some/random/link")),
            },
            // spaces after
            TestCase {
                input: String::from("/some/random/target /some/random/link      "),
                matches: true,
                target: Some(String::from("/some/random/target")),
                link: Some(String::from("/some/random/link")),
            },
            // spaces everywhere
            TestCase {
                input: String::from("     /some/random/target    /some/random/link      "),
                matches: true,
                target: Some(String::from("/some/random/target")),
                link: Some(String::from("/some/random/link")),
            },
            // target in quotes
            TestCase {
                input: String::from("\"/some/random/target\" /some/random/link"),
                matches: true,
                target: Some(String::from("\"/some/random/target\"")),
                link: Some(String::from("/some/random/link")),
            },
            // link in quotes
            TestCase {
                input: String::from("/some/random/target \"/some/random/link\""),
                matches: true,
                target: Some(String::from("/some/random/target")),
                link: Some(String::from("\"/some/random/link\"")),
            },
            // both in quotes
            TestCase {
                input: String::from("\"/some/random/target\" \"/some/random/link\""),
                matches: true,
                target: Some(String::from("\"/some/random/target\"")),
                link: Some(String::from("\"/some/random/link\"")),
            },
            // both in quotes with spaces
            TestCase {
                input: String::from(
                    "\"/some/random/target with spaces\" \"/some/random/link with spaces\"",
                ),
                matches: true,
                target: Some(String::from("\"/some/random/target with spaces\"")),
                link: Some(String::from("\"/some/random/link with spaces\"")),
            },
            // target contains double quote
            TestCase {
                input: String::from("/some/random/\"target /some/random/link"),
                matches: false,
                target: None,
                link: None,
            },
            // target contains double quote
            TestCase {
                input: String::from("/some/random/target /some/random/\"link"),
                matches: false,
                target: None,
                link: None,
            },
            // quotes within quotes
            TestCase {
                input: String::from("\"/some/random/\"target\" \"/some/random/\"link\""),
                matches: false,
                target: None,
                link: None,
            },
        ];

        for test_case in test_cases {
            let caps = SLS_SPEC_RE.captures(&test_case.input[..]);
            assert_eq!(
                caps.is_some(),
                test_case.matches,
                "Didn't match as expected for input '{}'",
                test_case.input
            );

            if let Some(caps) = caps {
                assert_eq!(&caps["target"], test_case.target.unwrap());
                assert_eq!(&caps["link"], test_case.link.unwrap());
            }
        }
    }
}
