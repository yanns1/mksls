//! Everything related to the app's CLI.

use clap::{crate_name, Parser};
use crossterm::style::Stylize;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version)]
#[clap(about = "Make symlinks specified in files.")]
#[clap(long_about = "Make symlinks specified in files.

This program makes the symlinks specified in files within DIR having the base FILENAME.
A given file contains zero or more symlink specifications, where a symlink specification is a line with the following format:
    <TARGET_PATH> <SYMLINK_PATH>
Notice the space in between.

There can be multiple spaces, but there needs to be at least one.
If a path contains a space, wrap it in double quotes.
For example, if <TARGET_PATH> contains a space, write this instead:
     \"<TARGET_PATH>\" <SYMLINK_PATH>
If you have a double quote in one of the paths... Change it!

By default, the program is interactive.
If no file is found where a given symlink is about to be made, the symlink will be made.
However, if a file is found, you will be asked to choose between:
    [s]kip : Don't create the symlink and move on to the next one.
    [S]kip all : [s]kip for the current symlink and all further symlink conflicting with an existing file.
    [b]ackup : Move the existing file in BACKUP_DIR, then make the current symlink.
    [B]ackup all : [b]ackup for the current symlink and all further symlink conflicting with an existing file.
    [o]verwrite : Overwrite the existing file with the symlink (beware data loss!)
    [O]verwrite all : [o]verwrite for the current symlink and all further symlink conflicting with an existing file.
However it can be made uninteractive by using one (and only one) of these options:
    --always-skip (equivalent to always selecting 's')
    --always-backup (equivalent to always selecting 'b')
There is no --always-overwrite for you to not regret it.

For each processed symlink specification, a line with the following format is printed:
    (<action>) <link> -> <target>
where <action> encodes what has been done for that symlink:
    . : Already existed, so has been skipped.
    d : Done. The symlink was successfully created.
    s : There was a conflict between the link and an existing file, and choose to [s]kip.
    b : There was a conflict between the link and an existing file, and choose to [b]ackup.
    o : There was a conflict between the link and an existing file, and choose to [o]verwrite.
(<link> and <target> are respectively the link and target of the symlink specification)")]
// NOTE: The path of the config file depends on `confy`, which uses `directories`.
// To keep up to date!
#[command(after_help = format!("{}
You can provide other default values for the options:
    --filename
    --backup-dir
    --always-skip
    --always-backup
in a TOML configuration file located at:
    (Linux) $XDG_CONFIG_HOME/<project_path> or .config/<project_path> if $XDG_CONFIG_HOME is not set
    (Mac) $HOME/Library/Application Support/<project_path>
where <project_path> is '{}/{}.toml'.

Note:
    - If you didn't write a config file yourself, one with the default values will automatically be written.
    - Paths in the config file should be absolute.
", "Configuration file:".bold().underlined(), crate_name!(), crate_name!()))]
/// Defines the command-line interface of the app.
///
/// This is a [`clap::Parser`].
///
/// # Examples
///
/// ```rust,no_run
/// use mksls::cli::Cli;
/// use clap::Parser;
///
/// let args = Cli::parse();
/// ```
pub struct Cli {
    /// The directory in which to scan for files specifying symlinks.
    #[clap(verbatim_doc_comment)]
    pub dir: PathBuf,

    /// The base (name + extension) of the file(s) specifying symlinks to make.
    ///
    /// By default, the name is "sls".
    /// If one is specified in the config file, it will be used instead.
    #[clap(verbatim_doc_comment)]
    #[arg(short, long)]
    pub filename: Option<String>,

    /// The backup directory in which to store the backed up files during execution.
    ///
    /// By default, it is set to:
    ///     (Linux) $XDG_CONFIG_HOME/mksls/backups/ or .config/mksls/backups/ if $XDG_CONFIG_HOME is not set
    ///     (Mac) $HOME/Library/Application Support/mksls/backups/
    #[clap(verbatim_doc_comment)]
    #[arg(short, long)]
    pub backup_dir: Option<PathBuf>,

    /// Always skip the symlinks conflicting with an existing file.
    ///
    /// This makes the program uninteractive.
    /// Of course, it can't be combined with --always-backup.
    #[clap(verbatim_doc_comment)]
    #[clap(long, conflicts_with = "always_backup")]
    pub always_skip: bool,

    /// Always backup the conflicting file before replacing it by the symlink.
    ///
    /// This makes the program uninteractive.
    /// Of course, it can't be combined with --always-skip.
    #[clap(verbatim_doc_comment)]
    #[clap(long, conflicts_with = "always_skip")]
    pub always_backup: bool,
}
