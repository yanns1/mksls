//! Where most of the app's logic resides.

use crate::dir::Dir;
use crate::line;
use crate::line::error::{NoMatchForLine, TargetDoesNotExistForLine};
use crate::line::{Invalid, LineType};
use crate::params::Params;
use anyhow::Context;
use crossterm::cursor;
use crossterm::style::Stylize;
use crossterm::terminal;
use crossterm::ExecutableCommand;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;

const INDENT: &str = "    ";
const ACTION_HELP: &str = "[s]kip : Don't create the symlink and move on to the next one.
[S]kip all : [s]kip for the current symlink and all further symlink conflicting with an existing file.
[b]ackup : Move the existing file in BACKUP_DIR, then make the current symlink.
[B]ackup all : [b]ackup for the current symlink and all further symlink conflicting with an existing file.
[o]verwrite : Overwrite the existing file with the symlink (beware data loss!)
[O]verwrite all : [o]verwrite for the current symlink and all further symlink conflicting with an existing file.";

/// The possible actions to take when a symlink about to be made conflicts with an existing file.
#[derive(Debug)]
enum Action {
    /// Don't make the symlink and move on.
    Skip,
    /// Backup the existing file, then make the symlink over the existing file.
    Backup,
    /// Make the symlink without backup, overwriting the existing file.
    Overwrite,
}

/// The engine of the program, where the app's pieces are glued together.
///
/// # Examples
///
/// ```rust,no_run
/// use clap::Parser;
/// use mksls::cfg::Config;
/// use mksls::cli::Cli;
/// use mksls::engine::Engine;
/// use mksls::params::Params;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let cli = Cli::parse();
///     let cfg: Config = confy::load("my_crate", "config")?;
///     let params = Params::new(cli, cfg)?;
///     let engine = Engine::new(params);
///
///     engine.run()?;
///
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct Engine {
    /// The action to be taken at any given time.
    action: Option<Action>,
    params: Params,
}

impl Engine {
    /// Creates an engine.
    ///
    /// # Parameters
    ///
    /// * `params` - Parameters to customize the engine's behavior.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use clap::Parser;
    /// use mksls::cfg::Config;
    /// use mksls::cli::Cli;
    /// use mksls::engine::Engine;
    /// use mksls::params::Params;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cli = Cli::parse();
    /// let cfg: Config = confy::load("my_crate", "config")?;
    /// let params = Params::new(cli, cfg)?;
    /// let engine = Engine::new(params);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(params: Params) -> Self {
        let mut action: Option<Action> = None;
        if params.always_skip {
            action = Some(Action::Skip);
        }
        if params.always_backup {
            action = Some(Action::Backup);
        }

        Self { action, params }
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
    /// * `writer` - Where to write feeback to.
    /// * `target` - Path to the target of the symlink.
    /// * `link` - Path to the symlink.
    fn skip<W: Write>(mut writer: W, target: &Path, link: &Path) -> anyhow::Result<()> {
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
    /// * `writer` - Where to write feedback to.
    /// * `backup_dir` - Path to backup directory.
    /// * `target` - Path to the target of the symlink.
    /// * `link` - Path to the symlink.
    ///
    /// # Errors
    ///
    /// Fails when:
    ///
    /// * The existing file fails to be backed up, i.e. fails to be moved
    ///   to the backup directory.
    /// * The symlink creation fails.
    /// * Writing into `writer` fails.
    ///
    /// These are `anyhow` errors, so most of the time, you just want to
    /// propagate them.
    fn backup<W: Write>(
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
    /// # Parameters
    ///
    /// * `writer` - Where to write feedback to.
    /// * `target` - Path to the target of the symlink.
    /// * `link` - Path to the symlink.
    ///
    /// # Errors
    ///
    /// Fails when:
    ///
    /// * The existing file fails to be removed.
    /// * The symlink creation fails.
    /// * Writing into `writer` fails.
    ///
    /// These are `anyhow` errors, so most of the time, you just want to
    /// propagate them.
    fn overwrite<W: Write>(mut writer: W, target: &Path, link: &Path) -> anyhow::Result<()> {
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

    /// Processes a symlink-specification file (`sls`).
    ///
    /// Reads `sls` line-by-line, creates the symlinks corresponding
    /// to the symlink specifications found.
    ///
    /// # Parameters
    ///
    /// * `sls` - Path to the symlink-specification file.
    ///
    /// # Errors
    ///
    /// Fails when:
    ///
    /// * Opening for read of `sls` fails.
    /// * Reading a line fails.
    /// * Processing a line fails (see [`Engine::process_line`]).
    ///
    /// These are `anyhow` errors, so most of the time, you just want to
    /// propagate them.
    fn process_file(&mut self, sls: PathBuf) -> anyhow::Result<()> {
        let file = fs::File::open(&sls).with_context(|| {
            format!("Tried to open {}, but unexpectedly failed.", sls.display())
        })?;
        let reader = io::BufReader::new(file);

        for (i, line) in reader.lines().enumerate() {
            let line_no = (i + 1) as u64;
            let line = line.with_context(|| {
                format!("Error reading line {} of file {}.", line_no, sls.display())
            })?;

            self.process_line(&sls, line_no, line)?;
        }

        Ok(())
    }

    /// Processes a `line` from a symlink-specification file.
    ///
    /// The processing depends on the [`line::LineType`] of `line`.
    ///
    /// * If [`line::LineType::Invalid`], errors with an informative message
    ///   for the user.
    /// * If [`line::LineType::Empty`], does nothing and returns.
    /// * If [`line::LineType::Comment`], does nothing and returns.
    /// * If [`line::LineType::SlsSpec`], tries to make the symlink specified,
    ///   or runs the interactive mahcinery in case there exists a conflicting file.
    ///   Finally, reports to the user what has been done.
    ///
    /// # Parameters
    ///
    /// * `sls` - Path to the symlink-specification file where `line` lives.
    /// * `line_no` - The line number of `line` in `sls`.
    /// * `line` - Contents of the line to process.
    ///
    /// # Errors
    ///
    /// Fails when:
    ///
    /// * `line` is of type [`line::LineType::Invalid`].
    /// * Symlink creation faiis.
    /// * Reading conflicting file/symlink fails.
    /// * Reading/writing from/to stdin/stdout fails.
    ///
    /// These are `anyhow` errors, so most of the time, you just want to
    /// propagate them.
    fn process_line(&mut self, sls: &Path, line_no: u64, line: String) -> anyhow::Result<()> {
        let mut stdout = io::stdout();

        match line::line_type(&line) {
            LineType::Empty | LineType::Comment => return Ok(()),
            LineType::Invalid(invalid) => match invalid {
                Invalid::NoMatch => {
                    return Err(NoMatchForLine {
                        file: sls.to_path_buf(),
                        line_no,
                    })?
                }
                Invalid::TargetDoesNotExist => {
                    return Err(TargetDoesNotExistForLine {
                        file: sls.to_path_buf(),
                        line_no,
                    })?
                }
            },
            LineType::SlsSpec { target, link } => {
                let link_str = link.to_string_lossy();
                if !link.is_symlink() && !link.exists() {
                    unix::fs::symlink(&target, &link).with_context(|| {
                        format!(
                            "Failed to create {} -> {}",
                            link_str,
                            target.to_string_lossy()
                        )
                    })?;
                    println!("(d) {} -> {}", link_str, target.to_string_lossy());
                } else {
                    if link.is_symlink()
                        && fs::read_link(&link).with_context(|| format!("A symlink of path {} already exists, but failed to read it to check if it is the one you want to create or not.
Nothing was done. Check for a problem and rerun this program.", link_str))?
                            == target
                    {
                        println!("{}", format!("(.) {} -> {}", link_str, target.to_string_lossy()).dark_grey());
                        return Ok(());
                    }

                    if let Some(ref action) = self.action {
                        match action {
                            Action::Skip => Engine::skip(stdout, &target, &link)?,
                            Action::Backup => {
                                Engine::backup(stdout, &self.params.backup_dir, &target, &link)?
                            }
                            Action::Overwrite => Engine::overwrite(stdout, &target, &link)?,
                        }
                        return Ok(());
                    }

                    let mut n_lines_to_overwrite = 0;
                    println!("(?) {} -> {}", link_str.red(), target.to_string_lossy());
                    n_lines_to_overwrite += 1;
                    println!("{INDENT}A file of path {} already exists.", link_str);
                    n_lines_to_overwrite += 1;

                    loop {
                        print!(
                            "{INDENT}[s]kip [S]kip all [b]ackup [B]ackup all [o]verwrite [O]verwrite all [h]elp: "
                        );
                        n_lines_to_overwrite += 1;
                        stdout
                            .flush()
                            .with_context(|| "Failed to completely write to stdout.")?;
                        let mut user_input = String::new();
                        io::stdin()
                            .read_line(&mut user_input)
                            .with_context(|| "Error reading your input!")?;
                        // Need this because the newline of Enter is included in the input
                        user_input.truncate(user_input.len() - 1);
                        match &user_input[..] {
                            "s" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                Engine::skip(stdout, &target, &link)?;
                                break;
                            }
                            "S" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                Engine::skip(stdout, &target, &link)?;
                                self.action = Some(Action::Skip);
                                break;
                            }
                            "b" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                Engine::backup(stdout, &self.params.backup_dir, &target, &link)?;
                                break;
                            }
                            "B" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                Engine::backup(stdout, &self.params.backup_dir, &target, &link)?;
                                self.action = Some(Action::Backup);
                                break;
                            }
                            "o" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                Engine::overwrite(stdout, &target, &link)?;
                                break;
                            }
                            "O" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                Engine::overwrite(stdout, &target, &link)?;
                                self.action = Some(Action::Overwrite);
                                break;
                            }
                            "h" => {
                                println!("{INDENT}----------");
                                n_lines_to_overwrite += 1;
                                for line in ACTION_HELP.lines() {
                                    println!("{INDENT}{}", line);
                                    n_lines_to_overwrite += 1;
                                }
                                println!("{INDENT}----------");
                                n_lines_to_overwrite += 1;
                                continue;
                            }
                            _ => {
                                println!("{INDENT}Wrong input! Valid outputs are: s, S, b, B, o, O, h. Try again:");
                                n_lines_to_overwrite += 1;
                                continue;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Runs the engine.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use clap::Parser;
    /// use mksls::cfg::Config;
    /// use mksls::cli::Cli;
    /// use mksls::engine::Engine;
    /// use mksls::params::Params;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cli = Cli::parse();
    /// let cfg: Config = confy::load("my_crate", "config")?;
    /// let params = Params::new(cli, cfg)?;
    /// let engine = Engine::new(params);
    ///
    /// engine.run()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn run(mut self) -> anyhow::Result<()> {
        let dir = Dir::build(self.params.dir.clone())?;
        for sls in dir.iter_on_sls_files(&self.params.filename[..]) {
            self.process_file(sls)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dir::Dir;
    use assert_fs::fixture::NamedTempFile;
    use assert_fs::fixture::TempDir;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use std::path::PathBuf;
    use std::str;

    #[test]
    fn skip_feedback_has_right_format() {
        let mut feedback = vec![];
        let target = PathBuf::from("/target");
        let link = PathBuf::from("/link");

        Engine::skip(&mut feedback, &target, &link)
            .expect("Expected to be able to write into `feedback`.");
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

        Engine::backup(&mut feedback, &backup_dir, &target, &conflicting_file)?;
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

        Engine::backup(&mut feedback, &backup_dir, &target, &conflicting_file)?;

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

        assert!(Engine::backup(&mut feedback, &backup_dir, &target, &conflicting_file).is_err());

        // Ensure deletion happens.
        backup_dir.close()?;
        target.close()?;

        Ok(())
    }
}
