//! Types and methods for easy iteration over files in a directory.

pub mod error;

use std::path::PathBuf;
use walkdir::WalkDir;

/// A wrapper around [`std::path::PathBuf`] that represents a valid directory.
///
/// Different iterators over the files of that directory are made available.
///
/// # Examples
///
/// Iterate over files in a directory:
///
/// ```rust,no_run
/// # use mksls::dir::Dir;
/// # use std::path::PathBuf;
/// #
/// let dir = Dir::build(PathBuf::from("/my/dir/path"))
///               .expect("Expected path to point to an existing directory.");
///
/// for file in dir.iter_on_files() {
///     println!("{}", file.to_string_lossy());
/// }
/// ```
///
/// Iterate over "symlink-specification" files in a directory:
///
/// ```rust,no_run
/// # use mksls::dir::Dir;
/// # use std::path::PathBuf;
/// #
/// let dir = Dir::build(PathBuf::from("/my/dir/path"))
///               .expect("Expected path to point to an existing directory.");
///
/// for sls_file in dir.iter_on_sls_files("sls") {
///     println!("{}", sls_file.to_string_lossy());
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Dir(PathBuf);

impl Dir {
    /// Creates a new [`Dir`], but can fail.
    ///
    /// If the input path does not point to an _existing directory_ an error
    /// is returned.
    ///
    /// # Parameters
    ///
    /// - `dir`: The path to the directory.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mksls::dir::Dir;
    /// # use std::path::PathBuf;
    /// #
    /// let dir = Dir::build(PathBuf::from("/my/dir/path"))
    ///               .expect("Expected path to point to an existing directory.");
    /// ```
    pub fn build(dir: PathBuf) -> Result<Self, error::DirDoesNotExist> {
        if !dir.is_dir() {
            return Err(error::DirDoesNotExist(dir));
        }
        Ok(Dir(dir))
    }

    /// Creates an iterator over the directory's files ([`DirFilesIter`]).
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mksls::dir::Dir;
    /// # use std::path::PathBuf;
    /// #
    /// let dir = Dir::build(PathBuf::from("/my/dir/path"))
    ///               .expect("Expected path to point to an existing directory.");
    ///
    /// for file in dir.iter_on_files() {
    ///     println!("{}", file.to_string_lossy());
    /// }
    /// ```
    pub fn iter_on_files(&self) -> DirFilesIter {
        DirFilesIter::new(self)
    }

    /// Creates an iterator over the directory's "symlink-specification" files ([`DirSlsFilesIter`]).
    ///
    /// # Parameters
    ///
    /// - `sls_filename`: The filename (name + extension) to look for.
    ///
    ///     Files with a filename equal to `sls_filename` will be considered
    ///     "symlink-specification" files.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # use mksls::dir::Dir;
    /// # use std::path::PathBuf;
    /// #
    /// let dir = Dir::build(PathBuf::from("/my/dir/path"))
    ///               .expect("Expected path to point to an existing directory.");
    ///
    /// for sls_file in dir.iter_on_sls_files("sls") {
    ///     println!("{}", sls_file.to_string_lossy());
    /// }
    /// ```
    pub fn iter_on_sls_files(&self, sls_filename: &str) -> DirSlsFilesIter {
        DirSlsFilesIter::new(self, sls_filename)
    }
}

/// An iterator over a directory's files.
pub struct DirFilesIter {
    walk_dir: Box<dyn Iterator<Item = PathBuf>>,
}

impl DirFilesIter {
    fn new(dir: &Dir) -> DirFilesIter {
        let walk_dir = WalkDir::new(&dir.0)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file() || entry.file_type().is_symlink())
            .map(|entry| entry.into_path());

        DirFilesIter {
            walk_dir: Box::new(walk_dir),
        }
    }
}

impl Iterator for DirFilesIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        self.walk_dir.next()
    }
}

/// An iterator over a directory's "symlink-specification" files.
pub struct DirSlsFilesIter {
    walk_dir: Box<dyn Iterator<Item = PathBuf>>,
}

impl DirSlsFilesIter {
    fn new(dir: &Dir, sls_filename: &str) -> DirSlsFilesIter {
        let sls_filename = String::from(sls_filename);

        let walk_dir = WalkDir::new(&dir.0)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file() || entry.file_type().is_symlink())
            .map(|entry| entry.into_path())
            .filter(move |file| match file.file_name() {
                Some(os_str) => os_str == &sls_filename[..],
                None => false,
            });

        DirSlsFilesIter {
            walk_dir: Box::new(walk_dir),
        }
    }
}

impl Iterator for DirSlsFilesIter {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        self.walk_dir.next()
    }
}

#[cfg(test)]
mod tests {
    use serial_test::serial;

    use super::*;
    use crate::utils;
    use core::panic;
    use std::fs;
    use std::os::unix;
    use std::path::PathBuf;

    fn get_tmp_dir() -> PathBuf {
        let mut tmp_dir = std::env::current_dir().unwrap();
        tmp_dir.push(".tmp");
        tmp_dir
    }

    fn mk_tmp_contents() -> Vec<PathBuf> {
        let mut contents: Vec<PathBuf> = vec![];

        // Create tmp dir.
        let tmp_dir = get_tmp_dir();
        if tmp_dir.exists() {
            // Remove tmp dir to recreate it later.
            // This is to ensure the tmp dir is in the state
            // we want it to.
            fs::remove_dir_all(&tmp_dir).unwrap();
        }
        fs::create_dir(&tmp_dir).unwrap();
        contents.push(tmp_dir.clone());

        // Make a few files...
        // Regular files
        let n_files = 5;
        for i in 1..n_files + 1 {
            let mut f = tmp_dir.clone();
            let filename = format!("f{}", i);
            f.push(&filename);
            if let Err(err) = fs::write(&f, filename) {
                panic!("{:?}", err);
            }
            contents.push(f);
        }
        let mut sls = tmp_dir.clone();
        sls.push("sls");
        let mut sl_spec_target = tmp_dir.clone();
        sl_spec_target.push("f2");
        let mut sl_spec_link = tmp_dir.clone();
        sl_spec_link.push("s2");
        let sl_spec = format!("{} {}", sl_spec_target.display(), sl_spec_link.display());
        let wrong_sl_spec = String::from("/wrong/\"target /wrong/\"link");
        let lines = vec![
            sl_spec,
            String::from(""),
            String::from("// a comment"),
            wrong_sl_spec,
        ];
        if let Err(err) = fs::write(&sls, lines.join("\n")) {
            panic!("{:?}", err);
        }
        contents.push(sls);

        // Symlinks
        let n_symlinks = 1;
        for i in 1..n_symlinks + 1 {
            let mut sl = tmp_dir.clone();
            sl.push(format!("s{}", i));

            let mut target = tmp_dir.clone();
            target.push(format!("f{}", i));

            if !sl.exists() {
                if let Err(err) = unix::fs::symlink(target, &sl) {
                    panic!("{:?}", err);
                }
            }

            contents.push(sl);
        }

        // Directories
        let n_dirs = 3;
        for i in 1..n_dirs + 1 {
            // Create the directory
            let mut d = tmp_dir.clone();
            d.push(format!("d{}", i));
            if !d.exists() {
                if let Err(err) = fs::create_dir(&d) {
                    panic!("{:?}", err);
                }
            }
            contents.push(d);
            // Add some files
            let n_files = 5;
            for j in 1..n_files + 1 {
                let mut f = tmp_dir.clone();
                f.push(format!("d{}", i));
                let filename = format!("d{}f{}", i, j);
                f.push(&filename);
                if let Err(err) = fs::write(&f, filename) {
                    panic!("{:?}", err);
                }
                contents.push(f);
            }
            // Add a sls file
            let mut sls = tmp_dir.clone();
            sls.push(format!("d{}/sls", i));
            let mut sl_spec_target = tmp_dir.clone();
            sl_spec_target.push(format!("f{}", i + 2));
            let mut sl_spec_link = tmp_dir.clone();
            sl_spec_link.push(format!("s{}", i + 2));
            let sl_spec = format!("{} {}", sl_spec_target.display(), sl_spec_link.display());
            if let Err(err) = fs::write(&sls, sl_spec) {
                panic!("{:?}", err);
            }
            contents.push(sls);
        }

        contents
    }

    #[test]
    fn dir_build_errors_when_dir_does_not_exist() {
        let mut path = get_tmp_dir();
        path.push("does_not_exist");

        let path_str = path.clone();
        let path_str = path_str
            .to_str()
            .expect("Expected only UTF-8 characters in the path.");

        let dir = Dir::build(path);

        assert!(
            dir.is_err(),
            "Expected Dir::build to error on {}.",
            path_str
        );
    }

    #[serial]
    #[test]
    fn dir_iter_on_files_successful() {
        let expected_files: Vec<PathBuf> = mk_tmp_contents()
            .into_iter()
            .filter(|path| path.is_file() || path.is_symlink())
            .collect();

        let tmp_dir = get_tmp_dir();
        let tmp_dir = Dir::build(tmp_dir).expect("tmp_dir should exist at this point");
        let files_it = tmp_dir.iter_on_files();
        let files: Vec<PathBuf> = files_it.collect();
        assert!(utils::tests::vec_are_equal(&files, &expected_files));
    }

    #[serial]
    #[test]
    fn dir_iter_on_sls_files_successful() {
        let sls_filename = "sls";

        let expected_sls_files: Vec<PathBuf> = mk_tmp_contents()
            .into_iter()
            .filter(|path| path.is_file() || path.is_symlink())
            .filter(|path| match path.file_name() {
                Some(os_str) => os_str == sls_filename,
                None => false,
            })
            .collect();

        let tmp_dir = get_tmp_dir();
        let tmp_dir = Dir::build(tmp_dir).expect("tmp_dir should exist at this point");
        let sls_files_it = tmp_dir.iter_on_sls_files(sls_filename);
        let sls_files: Vec<PathBuf> = sls_files_it.collect();
        assert!(utils::tests::vec_are_equal(&sls_files, &expected_sls_files));
    }
}
