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
    Skip,
    Backup,
    Overwrite,
}

/// The heart of the program, where the action takes place.
#[derive(Debug)]
pub struct Engine {
    /// The action to be taken at any given time.
    action: Option<Action>,
    params: Params,
}

impl Engine {
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

    fn skip(&self, target: &Path, link: &Path) {
        println!(
            "{}",
            format!(
                "(s) {} -> {}",
                link.to_string_lossy(),
                target.to_string_lossy()
            )
            .dark_blue()
        );
    }

    fn backup(&self, target: &Path, link: &Path) -> anyhow::Result<()> {
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

        let mut backup = self.params.backup_dir.clone();
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

        println!(
            "{}",
            format!(
                "(b) {} -> {}",
                link.to_string_lossy(),
                target.to_string_lossy()
            )
            .dark_green()
        );

        Ok(())
    }

    fn overwrite(&self, target: &Path, link: &Path) -> anyhow::Result<()> {
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

        println!(
            "{}",
            format!(
                "(o) {} -> {}",
                link.to_string_lossy(),
                target.to_string_lossy()
            )
            .dark_red()
        );

        Ok(())
    }

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
                            Action::Skip => self.skip(&target, &link),
                            Action::Backup => self.backup(&target, &link)?,
                            Action::Overwrite => self.overwrite(&target, &link)?,
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
                                self.skip(&target, &link);
                                break;
                            }
                            "S" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                self.skip(&target, &link);
                                self.action = Some(Action::Skip);
                                break;
                            }
                            "b" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                self.backup(&target, &link)?;
                                break;
                            }
                            "B" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                self.backup(&target, &link)?;
                                self.action = Some(Action::Backup);
                                break;
                            }
                            "o" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                self.overwrite(&target, &link)?;
                                break;
                            }
                            "O" => {
                                stdout
                                    .execute(cursor::MoveUp(n_lines_to_overwrite))?
                                    .execute(terminal::Clear(
                                        terminal::ClearType::FromCursorDown,
                                    ))?;
                                self.overwrite(&target, &link)?;
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

    pub fn run(mut self) -> anyhow::Result<()> {
        let dir = Dir::build(self.params.dir.clone())?;
        for sls in dir.iter_on_sls_files(&self.params.filename[..]) {
            self.process_file(sls)?;
        }

        Ok(())
    }
}
