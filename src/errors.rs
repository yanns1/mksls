use core::fmt;
use std::{error::Error, path::PathBuf};

#[derive(Debug)]
pub struct DirDoesNotExist {
    pub dir: PathBuf,
}

impl DirDoesNotExist {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }
}

impl Error for DirDoesNotExist {}

impl fmt::Display for DirDoesNotExist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The directory {} does not exist.",
            self.dir
                .to_str()
                .expect("Expected only UTF-8 characters in the path.")
        )
    }
}

#[derive(Debug)]
pub struct DirCreationFailed {
    pub dir: PathBuf,
    pub err: Box<dyn Error>,
}

impl DirCreationFailed {
    pub fn new(dir: PathBuf, err: Box<dyn Error>) -> Self {
        Self { dir, err }
    }
}

impl Error for DirCreationFailed {}

impl fmt::Display for DirCreationFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The directory {} does not exist, and its creation failed for the following reason: {}",
            self.err,
            self.dir
                .to_str()
                .expect("Expected only UTF-8 characters in the path.")
        )
    }
}
