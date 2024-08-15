//! Errors corresponding [`crate::line::Invalid`].

use core::fmt;
use std::{error, fmt::Debug, path::PathBuf};

#[derive(Debug)]
/// An error for when a line (in a symlink-specification file) does not match the
/// specification format.
pub struct NoMatchForLine {
    /// The path of the symlink-specification file.
    pub file: PathBuf,
    /// The line number of the problematic line.
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
/// An error for when a line (in a symlink-specification file) matches the
/// specification format, but the target path points to a non-existing file.
pub struct TargetDoesNotExistForLine {
    /// The path of the symlink-specification file.
    pub file: PathBuf,
    /// The line number of the problematic line.
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
