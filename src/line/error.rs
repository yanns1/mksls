use core::fmt;
use std::{error, fmt::Debug, path::PathBuf};

#[derive(Debug)]
pub struct InvalidLine {
    pub file: PathBuf,
    pub line_no: u64,
}

impl fmt::Display for InvalidLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid line in {}, line number {}.",
            self.file.display(),
            self.line_no,
        )
    }
}

impl error::Error for InvalidLine {}
