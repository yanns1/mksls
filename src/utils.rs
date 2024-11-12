use anyhow::Context;
use crossterm::style::Stylize;
use std::fs;
use std::io::Write;
use std::os::unix;
use std::path::Path;

pub fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}

/// Skips symlink creation when conflict encountered, i.e. when `link`
/// already points to a file.
///
/// Does nothing apart from writing feedback into `writer` in the form of:
///
/// ```text
/// (s) <link> -> <target>
/// ```
///
/// in dark blue.
///
/// # Parameters
///
/// - `writer`: Where to write feeback to.
/// - `target`: Path to the target of the symlink.
/// - `link`: Path to the symlink.
pub fn skip<W: Write>(mut writer: W, target: &Path, link: &Path) -> anyhow::Result<()> {
    writeln!(
        writer,
        "{}",
        format!(
            "(s) {} -> {}",
            link.to_string_lossy(),
            target.to_string_lossy()
        )
        .dark_blue()
    )?;

    Ok(())
}

/// Backs up the existing file at path `link`, then makes the symlink
/// at path `link`, pointing to `target`.
///
/// Finally, writes feeback into `writer` in the form of:
///
/// ```text
/// (b) <link> -> <target>
/// ```
///
/// in dark green.
///
/// # Parameters
///
/// - `writer`: Where to write feedback to.
/// - `backup_dir`: Path to backup directory.
/// - `target`: Path to the target of the symlink.
/// - `link`: Path to the symlink.
///
/// # Errors
///
/// Fails when:
///
/// - The existing file fails to be backed up, i.e. fails to be moved
///   to the backup directory.
/// - The symlink creation fails.
/// - Writing into `writer` fails.
///
/// These are `anyhow` errors, so most of the time, you just want to
/// propagate them.
pub fn backup<W: Write>(
    mut writer: W,
    backup_dir: &Path,
    target: &Path,
    link: &Path,
) -> anyhow::Result<()> {
    let mut new_name;
    match link.file_stem() {
        Some(file_stem) => {
            new_name = format!(
                "{}_backup_{}",
                file_stem.to_string_lossy(),
                chrono::Local::now().to_rfc3339()
            );
            if let Some(extension) = link.extension() {
                new_name.push_str(&format!(".{}", extension.to_string_lossy()));
            }
        }
        None => {
            new_name = String::from(".");
            if let Some(extension) = link.extension() {
                new_name.push_str(&format!(
                    "{}_backup_{}",
                    extension.to_string_lossy(),
                    chrono::Local::now().to_rfc3339()
                ));
            }
        }
    }

    let mut backup = backup_dir.to_path_buf();
    backup.push(new_name);

    fs::rename(link, &backup).with_context(|| {
        format!(
            "Failed to backup! Couldn't move {} to {}",
            link.display(),
            backup.display()
        )
    })?;

    unix::fs::symlink(target, link).with_context(|| {
        format!(
            "Failed to create {} -> {}",
            link.to_string_lossy(),
            target.to_string_lossy()
        )
    })?;

    writeln!(
        writer,
        "{}",
        format!(
            "(b) {} -> {}",
            link.to_string_lossy(),
            target.to_string_lossy()
        )
        .dark_green()
    )?;

    Ok(())
}

/// Overwrites existing file at path `link` by making a symlink
/// at path `link` (pointing to `target`) without backup.
///
/// Finally, writes feeback into `writer` in the form of:
///
/// ```text
/// (o) <link> -> <target>
/// ```
///
/// in dark red.
///
/// # Parameters
///
/// - `writer`: Where to write feedback to.
/// - `target`: Path to the target of the symlink.
/// - `link`: Path to the symlink.
///
/// # Errors
///
/// Fails when:
///
/// - The existing file fails to be removed.
/// - The symlink creation fails.
/// - Writing into `writer` fails.
///
/// These are `anyhow` errors, so most of the time, you just want to
/// propagate them.
pub fn overwrite<W: Write>(mut writer: W, target: &Path, link: &Path) -> anyhow::Result<()> {
    if link.is_dir() {
        fs::remove_dir_all(link)
            .with_context(|| format!("Failed to remove current directory {} to then make the symlink with the same path.", link.to_string_lossy()))?;
    } else {
        fs::remove_file(link).with_context(|| {
            format!(
                "Failed to remove current file {} to then make the symlink with the same path.",
                link.to_string_lossy()
            )
        })?;
    }

    unix::fs::symlink(target, link).with_context(|| {
        format!(
            "Failed to create {} -> {}",
            link.to_string_lossy(),
            target.to_string_lossy()
        )
    })?;

    writeln!(
        writer,
        "{}",
        format!(
            "(o) {} -> {}",
            link.to_string_lossy(),
            target.to_string_lossy()
        )
        .dark_red()
    )?;

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::dir::Dir;
    use assert_fs::fixture::NamedTempFile;
    use assert_fs::fixture::TempDir;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use std::path::PathBuf;
    use std::str;

    pub fn vec_are_equal<T: Eq>(v1: &Vec<T>, v2: &Vec<T>) -> bool {
        v1.len() == v2.len() && v1.iter().fold(true, |acc, el| acc && v2.contains(el))
    }

    #[test]
    fn skip_feedback_has_right_format() {
        let mut feedback = vec![];
        let target = PathBuf::from("/target");
        let link = PathBuf::from("/link");

        skip(&mut feedback, &target, &link).expect("Expected to be able to write into `feedback`.");
        let feedback = str::from_utf8(&feedback[..]).expect("Should be valid utf-8 characters.");

        let expected_feedback = format!(
            "(s) {} -> {}",
            link.to_string_lossy(),
            target.to_string_lossy()
        )
        .dark_blue()
        .to_string();

        assert!(
            feedback.contains(&expected_feedback[..]),
            "Expected '{}' to contain '{}'.",
            feedback,
            expected_feedback,
        );
    }

    #[test]
    fn backup_feedback_has_right_format() -> Result<(), Box<dyn std::error::Error>> {
        let mut feedback = vec![];
        let backup_dir = TempDir::new()?;
        let target = NamedTempFile::new("target")?;
        target.touch()?;
        let conflicting_file = NamedTempFile::new("conflicting_file")?;
        conflicting_file.write_str("Contents of conflicting file.")?;

        backup(&mut feedback, &backup_dir, &target, &conflicting_file)?;
        let feedback = str::from_utf8(&feedback[..]).expect("Should be valid utf-8 characters.");

        let expected_feedback = format!(
            "(b) {} -> {}",
            conflicting_file.to_string_lossy(),
            target.to_string_lossy()
        )
        .dark_green()
        .to_string();

        assert!(
            feedback.contains(&expected_feedback[..]),
            "Expected '{}' to contain '{}'.",
            feedback,
            expected_feedback,
        );

        // Ensure deletion happens.
        backup_dir.close()?;
        target.close()?;
        conflicting_file.close()?;

        Ok(())
    }

    #[test]
    fn backup_backs_up_file_as_expected() -> Result<(), Box<dyn std::error::Error>> {
        let mut feedback = vec![];
        let backup_dir = TempDir::new()?;
        let dir = TempDir::new()?;
        let conflicting_file_name = "link";
        let conflicting_file = dir.child(conflicting_file_name);
        let conflicting_file_contents = "Contents of conflicting file.";
        conflicting_file.write_str(conflicting_file_contents)?;
        let target = NamedTempFile::new("target")?;
        target.touch()?;

        backup(&mut feedback, &backup_dir, &target, &conflicting_file)?;

        // Check that a file containing the name of `conflicting_file` exists in `backup_dir`.
        let d = Dir::build(backup_dir.to_path_buf())
            .expect("Path of `backup_dir` should be valid at this point.");
        let mut at_least_one_file_containing_conflicting_file_name = false;
        let mut backup_file: Option<PathBuf> = None;
        for file in d.iter_on_files() {
            if file
                .file_name()
                .unwrap()
                .to_string_lossy()
                .contains(conflicting_file_name)
            {
                backup_file = Some(file.clone());
                at_least_one_file_containing_conflicting_file_name = true;
            }
        }
        assert!(at_least_one_file_containing_conflicting_file_name);

        // Check that `backup_file` has the contents of the conflicting file.
        let backup_file = backup_file.expect(
            "Should have found a file containing the name of `conflicting_file` in `backup_dir`.",
        );
        let backup_file_contents = std::fs::read_to_string(backup_file)?;
        assert_eq!(backup_file_contents, conflicting_file_contents);

        // Ensure deletion happens.
        backup_dir.close()?;
        dir.close()?;
        target.close()?;

        Ok(())
    }

    #[test]
    fn backup_fails_when_no_conflicting_file() -> Result<(), Box<dyn std::error::Error>> {
        let mut feedback = vec![];
        let backup_dir = TempDir::new()?;
        // Do not touch or write to `conflicting_file` so that it doesn't actually exist in the file system.
        let conflicting_file = NamedTempFile::new("conflicting_file")?;
        let target = NamedTempFile::new("target")?;

        assert!(backup(&mut feedback, &backup_dir, &target, &conflicting_file).is_err());

        // Ensure deletion happens.
        backup_dir.close()?;
        target.close()?;

        Ok(())
    }

    #[test]
    fn overwrite_feedback_has_right_format() -> Result<(), Box<dyn std::error::Error>> {
        let mut feedback = vec![];
        let backup_dir = TempDir::new()?;
        let target = NamedTempFile::new("target")?;
        target.touch()?;
        let conflicting_file = NamedTempFile::new("conflicting_file")?;
        conflicting_file.write_str("Contents of conflicting file.")?;

        overwrite(&mut feedback, &target, &conflicting_file)?;
        let feedback = str::from_utf8(&feedback[..]).expect("Should be valid utf-8 characters.");

        let expected_feedback = format!(
            "(o) {} -> {}",
            conflicting_file.to_string_lossy(),
            target.to_string_lossy()
        )
        .dark_red()
        .to_string();

        assert!(
            feedback.contains(&expected_feedback[..]),
            "Expected '{}' to contain '{}'.",
            feedback,
            expected_feedback,
        );

        // Ensure deletion happens.
        backup_dir.close()?;
        target.close()?;
        conflicting_file.close()?;

        Ok(())
    }

    #[test]
    fn overwrite_overwrites_file_as_expected() -> Result<(), Box<dyn std::error::Error>> {
        let mut feedback = vec![];
        let conflicting_file_name = "link";
        let conflicting_file = NamedTempFile::new(conflicting_file_name)?;
        let conflicting_file_contents = "Contents of conflicting file.";
        conflicting_file.write_str(conflicting_file_contents)?;
        let target = NamedTempFile::new("target")?;
        target.touch()?;

        overwrite(&mut feedback, &target, &conflicting_file)?;

        // Check that a symlink to `target` exists in place of `conflicting_file`.
        assert!(predicate::path::is_symlink().eval(&conflicting_file));
        assert_eq!(
            std::fs::canonicalize(&conflicting_file).unwrap(),
            target.path()
        );

        // Ensure deletion happens.
        target.close()?;
        conflicting_file.close()?;

        Ok(())
    }

    #[test]
    fn overwrite_fails_when_no_conflicting_file() -> Result<(), Box<dyn std::error::Error>> {
        let mut feedback = vec![];
        // Do not touch or write to `conflicting_file` so that it doesn't actually exist in the file system.
        let conflicting_file = NamedTempFile::new("conflicting_file")?;
        let target = NamedTempFile::new("target")?;
        target.touch()?;

        assert!(overwrite(&mut feedback, &target, &conflicting_file).is_err());

        // Ensure deletion happens.
        conflicting_file.close()?;
        target.close()?;

        Ok(())
    }
}
