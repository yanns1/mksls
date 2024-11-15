//! Types and functions for parsing a line in a symlink-specification file and extracting
//! the relevant contents.

use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

lazy_static! {
    /// A regex to parse a line expected to contain a symlink specification.
    pub static ref SLS_SPEC_RE: Regex =
        Regex::new(r#"^\s*(?<target>[^\s"]+|"[^"]+")\s+(?<link>[^\s"]+|"[^"]+")\s*$"#).unwrap();
}

/// Ways a line expected to contain a symlink specification can be invalid.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Invalid {
    /// When the line doesn't match [`struct@SLS_SPEC_RE`].
    NoMatch,
    /// When the line matches [`struct@SLS_SPEC_RE`] but the target of the symlink doesn't exist.
    TargetDoesNotExist,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Types of lines that can be encountered during parsing.
pub enum LineType {
    /// A line containing an invalid symlink specification.
    Invalid(Invalid),
    /// An empty line.
    Empty,
    /// A line containing a comment.
    Comment,
    /// A line containing a valid symlink specification.
    SlsSpec {
        /// The path of the symlink's target.
        target: PathBuf,
        /// The path of the symlink.
        link: PathBuf,
    },
}

/// Returns the type of a line.
///
/// # Parameters
///
/// * `line` - The line for which to figure out the type.
///
/// # Examples
///
/// ```rust
/// use mksls::line;
/// use mksls::line::LineType;
/// use mksls::line::Invalid;
///
/// let invalid_line = "/wrong/\"target /wrong/\"link";
/// assert_eq!(line::line_type(invalid_line), LineType::Invalid(Invalid::NoMatch));
///
/// let empty_line = "";
/// assert_eq!(line::line_type(empty_line), LineType::Empty);
///
/// let comment_line = "// A comment.";
/// assert_eq!(line::line_type(comment_line), LineType::Comment);
///
/// let valid_line = "/home/my_user/.dotfiles/my_program/config /home/my_user/.config/my_program_config";
/// // It actually isn't quite valid because the target does not exist.
/// // The format is correct however.
/// assert_eq!(line::line_type(valid_line), LineType::Invalid(Invalid::TargetDoesNotExist));
/// ```
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
                if !target.exists() {
                    return LineType::Invalid(Invalid::TargetDoesNotExist);
                }
                let mut link = PathBuf::new();
                link.push(&caps["link"]);
                LineType::SlsSpec { target, link }
            }
            None => LineType::Invalid(Invalid::NoMatch),
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
