use std::path::PathBuf;

use anyhow::anyhow;

use crate::{Cli, Config};

/// An aggregation of configurations coming from the CLI ([`Cli`]) and the configuration file
/// ([`Config`]).
/// A configuration coming from the CLI always takes precedence.
/// A configuration coming from the configuration file is applied only when the equivalent is not
/// specified at the CLI level.
#[derive(Debug)]
pub struct Params {
    /// Same as [`Cli::dir`].
    pub dir: PathBuf,

    /// Same as [`Cli::filename`].
    pub filename: String,

    /// Same as [`Cli::backup_dir`].
    pub backup_dir: PathBuf,

    /// Same as [`Cli::always_skip`].
    pub always_skip: bool,

    /// Same as [`Cli::always_backup`].
    pub always_backup: bool,
}

impl Params {
    pub fn new(cli: Cli, cfg: Config) -> anyhow::Result<Self> {
        // backup_dir in Config should be absolute
        if cfg.backup_dir.is_relative() {
            return Err(anyhow!("Got a relative path for backup_dir in the configuration file, but backup_dir should be absolute."));
        }

        // Enforce mutual exclusivity of always_skip and always_backup for Config
        // (no need for Cli if `conflicts` is used)
        assert!(!(cli.always_skip && cli.always_backup));
        if cfg.always_skip && cfg.always_backup {
            return Err(anyhow!("Got always_skip and always_backup set to true in the configuration file, but only one of them can be true."));
        }

        let filename = cli.filename.unwrap_or(cfg.filename);

        let backup_dir = cli.backup_dir.unwrap_or(cfg.backup_dir);

        let mut always_skip = cli.always_skip;
        let mut always_backup = cli.always_backup;
        if !(always_skip || always_backup) {
            always_skip = cfg.always_skip;
            always_backup = cfg.always_backup;
        }

        Ok(Params {
            dir: cli.dir,
            filename,
            backup_dir,
            always_skip,
            always_backup,
        })
    }
}
