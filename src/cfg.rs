use clap::crate_name;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
/// The struct that defines the configuration file entries.
/// It is then used with [`confy::load()`].
pub struct Config {
    /// Same as [`crate::cli::Cli::filename`].
    pub filename: String,

    /// Same as [`crate::cli::Cli::backup_dir`].
    pub backup_dir: PathBuf,

    /// Same as [`crate::cli::Cli::always_skip`].
    pub always_skip: bool,

    /// Same as [`crate::cli::Cli::always_backup`].
    pub always_backup: bool,
}

impl std::default::Default for Config {
    fn default() -> Self {
        Self {
            filename: String::from("sls"),
            backup_dir: confy::get_configuration_file_path(crate_name!(), crate_name!())
                .unwrap()
                .parent()
                .unwrap()
                .join("backups/"),
            always_skip: false,
            always_backup: false,
        }
    }
}
