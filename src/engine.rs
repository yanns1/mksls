//! Where most of the app's logic resides.

use crate::dir::Dir;
use crate::line;
use crate::line::{Invalid, LineType};
use crate::params::Params;
use crate::prompt;
use crate::prompt::AlreadyExistPromptOptions;
use crate::utils;
use anyhow::Context;
use crossterm::style::Stylize;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::io::BufRead;
use std::os::unix;
use std::path::Path;
use std::path::PathBuf;

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
    ///   or runs the interactive machinery in case there exists a conflicting file.
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
        let stdout = io::stdout();
        match line::line_type(&line) {
            LineType::Empty | LineType::Comment => {
                return Ok(());
            }

            LineType::Invalid(invalid) => {
                let err_mess = match invalid {
                    Invalid::NoMatch => format!(
                        "Invalid line in {}, line number {}.
    Can't match up against the symlink specification format.",
                        sls.to_string_lossy(),
                        line_no
                    ),
                    Invalid::TargetDoesNotExist => format!(
                        "Invalid line in {}, line number {}.
    The target does not exist.",
                        sls.to_string_lossy(),
                        line_no
                    ),
                };
                prompt::error_prompt(&err_mess)?;
            }

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
                    return Ok(());
                }

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
                        Action::Skip => utils::skip(stdout, &target, &link)?,
                        Action::Backup => {
                            utils::backup(stdout, &self.params.backup_dir, &target, &link)?
                        }
                        Action::Overwrite => utils::overwrite(stdout, &target, &link)?,
                    }
                    return Ok(());
                }

                match prompt::already_exist_prompt(&target.to_string_lossy(), &link_str)? {
                    AlreadyExistPromptOptions::Skip => {
                        utils::skip(stdout, &target, &link)?;
                    }
                    AlreadyExistPromptOptions::AlwaysSkip => {
                        utils::skip(stdout, &target, &link)?;
                        self.action = Some(Action::Skip);
                    }
                    AlreadyExistPromptOptions::Backup => {
                        utils::backup(stdout, &self.params.backup_dir, &target, &link)?
                    }
                    AlreadyExistPromptOptions::AlwaysBackup => {
                        utils::backup(stdout, &self.params.backup_dir, &target, &link)?;
                        self.action = Some(Action::Backup);
                    }
                    AlreadyExistPromptOptions::Overwrite => {
                        utils::overwrite(stdout, &target, &link)?;
                    }
                    AlreadyExistPromptOptions::AlwaysOverwrite => {
                        utils::overwrite(stdout, &target, &link)?;
                        self.action = Some(Action::Overwrite);
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
