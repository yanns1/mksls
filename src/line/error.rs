use core::fmt;
use std::{error, fmt::Debug, path::PathBuf};

#[derive(Debug)]
pub struct NoMatchForLine {
    pub file: PathBuf,
    pub line_no: u64,
}

impl fmt::Display for NoMatchForLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid line in {}, line number {}.
Can't match up against the symlink specification format.",
            self.file.display(),
            self.line_no,
        )
    }
}

impl error::Error for NoMatchForLine {}

#[derive(Debug)]
pub struct TargetDoesNotExistForLine {
    pub file: PathBuf,
    pub line_no: u64,
}

impl fmt::Display for TargetDoesNotExistForLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Invalid line in {}, line number {}.
The target does not exist.",
            self.file.display(),
            self.line_no,
        )
    }
}

impl error::Error for TargetDoesNotExistForLine {}
