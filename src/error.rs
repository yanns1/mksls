use core::fmt;
use std::{error, fmt::Debug, io, path::PathBuf};

#[derive(Debug)]
pub struct DirDoesNotExist(pub PathBuf);

#[derive(Debug)]
pub struct DirCreationFailed(pub PathBuf, pub io::Error);

impl fmt::Display for DirDoesNotExist {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The directory {} does not exist.",
            self.0
                .to_str()
                .expect("Expected only UTF-8 characters in the path.")
        )
    }
}

impl error::Error for DirDoesNotExist {}

impl fmt::Display for DirCreationFailed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The creation of directory {} failed.
The underlying error is:
{:4?}",
            self.0
                .to_str()
                .expect("Expected only UTF-8 characters in the path."),
            self.1
        )
    }
}

impl error::Error for DirCreationFailed {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // The cause is the underlying implementation error type. Is implicitly
        // cast to the trait object `&error::Error`. This works because the
        // underlying type already implements the `Error` trait.
        Some(&self.1)
    }
}
