mod errors;

use clap::ArgAction;
use clap::Parser;
use colored::Colorize;
use errors::DirDoesNotExist;
use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::i64;
use std::path::PathBuf;

use crate::errors::DirCreationFailed;

const APP_NAME: &str = "mksls";

#[derive(Parser, Debug)]
#[command(name = APP_NAME)]
#[command(version)]
#[command(about = "Make symlinks specified in files.")]
#[command(long_about = "Make symlinks specified in files.
--------
This program makes the symlinks specified in files within DIR having the base FILE.
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
However it can be made uninteractive by using one (and only) of these options:
    --always-skip (equivalent to always selecting 's')
    --always-backup (equivalent to always selecting 'b')
There is no --always-overwrite for you to not regret it.
")]
// The path of the config file depends on `confy`, which uses `directories`.
// To keep up to date!
#[command(after_help = format!("{}
You can provide other default values for the options:
    --filename
    --backup-dir
    --depth
in a TOML configuration file located at:
    (Linux) $XDG_CONFIG_HOME/_project_path_ or .config/_project_path_ if $XDG_CONFIG_HOME is not set
    (Mac) $HOME/Library/Application Support/_project_path_
where _project_path_ is '{}/{}.toml'.

Note:
    - If you didn't write a config file yourself, one with the default values will automatically be written.
    - Paths in the config file should be absolute.
", "Configuration file:".bold().underline(), APP_NAME, APP_NAME))]
struct Cli {
    /// The directory in which to scan for files specifying symlinks.
    #[clap(verbatim_doc_comment)]
    dir: String,

    /// The base (name + extension) of the file(s) specifying symlinks to make.
    ///
    /// By default, the name is "sls".
    /// If one is specified in the config file, it will be used instead.
    #[clap(verbatim_doc_comment)]
    #[arg(short, long)]
    filename: Option<String>,

    /// The depth up to which files specifying symlinks to make will be considered.
    ///     
    /// By default, depth is unlimited, meaning the program will search as deep as
    /// it can in the input directory.
    /// If its value is specified in the config, it will be used instead (set to a
    /// negative integer, say -1, to mean "unlimited").
    #[clap(verbatim_doc_comment)]
    #[arg(short, long)]
    depth: Option<i64>,

    /// The backup directory in which to store the backed up files during execution.
    ///
    /// By default, it is set to:
    ///     (Linux) $XDG_CONFIG_HOME/mksls/backups/ or .config/mksls/backups/ if $XDG_CONFIG_HOME is not set
    ///     (Mac) $HOME/Library/Application Support/mksls/backups/
    #[clap(verbatim_doc_comment)]
    #[arg(short, long)]
    backup_dir: Option<String>,

    /// Always skip the symlinks conflicting with an existing file.
    ///
    /// This make the program uninteractive.
    /// Of course, it can't be combined with --always-backup.
    #[clap(verbatim_doc_comment)]
    #[clap(long, action=ArgAction::SetTrue, num_args = 0, conflicts_with = "always_backup")]
    always_skip: Option<bool>,

    /// Always backup the conflicting file before replacing it by the symlink.
    ///
    /// This make the program uninteractive.
    /// Of course, it can't be combined with --always-skip.
    #[clap(verbatim_doc_comment)]
    #[clap(long, action=ArgAction::SetTrue, num_args = 0, conflicts_with = "always_skip")]
    always_backup: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    filename: String,
    backup_dir: String,
    depth: i64,
    always_skip: bool,
    always_backup: bool,
}

impl ::std::default::Default for Config {
    fn default() -> Self {
        Self {
            filename: String::from("sls"),
            backup_dir: String::from(
                confy::get_configuration_file_path(APP_NAME, APP_NAME)
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("backups/")
                    .to_str()
                    .unwrap(),
            ),
            depth: -1,
            always_skip: false,
            always_backup: false,
        }
    }
}

#[derive(Debug)]
struct Params {
    dir: PathBuf,
    filename: String,
    backup_dir: PathBuf,
    depth: i64,
    always_skip: bool,
    always_backup: bool,
}

impl Params {
    fn new(cli: Cli, cfg: Config) -> Self {
        let mut dir = PathBuf::new();
        dir.push(cli.dir.as_str());

        let filename = cli.filename.unwrap_or(cfg.filename);

        let bd = cli.backup_dir.unwrap_or(cfg.backup_dir);
        let mut backup_dir = PathBuf::new();
        backup_dir.push(bd);

        let depth = cli.depth.unwrap_or(cfg.depth);

        let always_skip = cli.always_skip.unwrap_or(cfg.always_skip);

        let always_backup = cli.always_backup.unwrap_or(cfg.always_backup);

        Params {
            dir,
            filename,
            backup_dir,
            depth,
            always_skip,
            always_backup,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let cfg: Config = confy::load(APP_NAME, APP_NAME)?;

    let params = Params::new(cli, cfg);
    if !params.dir.is_dir() {
        return Err(Box::new(DirDoesNotExist::new(params.dir)));
    }
    if !params.backup_dir.is_dir() {
        if let Err(err) = fs::create_dir_all(params.backup_dir.as_path()) {
            return Err(Box::new(DirCreationFailed::new(
                params.backup_dir,
                Box::new(err),
            )));
        }
    }

    println!("{:?}", params);

    Ok(())
}
