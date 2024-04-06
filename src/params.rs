use std::path::PathBuf;

use anyhow::anyhow;

use crate::{Cli, Config};

#[derive(Debug)]
pub struct Params {
    pub dir: PathBuf,
    pub filename: String,
    pub backup_dir: PathBuf,
    pub always_skip: bool,
    pub always_backup: bool,
}

impl Params {
    pub fn new(cli: Cli, cfg: Config) -> anyhow::Result<Self> {
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
