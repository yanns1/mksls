//! Where the arguments/configurations for the app, coming from different sources,
//! are validated and united into a single data structure easy to use across the
//! codebase.

use crate::cfg::Config;
use crate::cli::Cli;
use anyhow::anyhow;
use std::path::PathBuf;

/// An aggregation of configurations coming from the CLI ([`Cli`]) and the configuration file
/// ([`Config`]), with verification of the validity.
///
/// A configuration coming from the CLI always takes precedence.
/// A configuration coming from the configuration file is applied only when the equivalent is not
/// specified at the CLI level.
///
/// # Examples
///
/// ```rust,no_run
/// use clap::Parser;
/// use mksls::cfg::Config;
/// use mksls::cli::Cli;
/// use mksls::params::Params;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let cli = Cli::parse();
///     let cfg: Config = confy::load("my_crate", "config")?;
///
///     let params = Params::new(cli, cfg)?;
///     println!("{:?}", params);
///
///     // ...
///
///     Ok(())
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Params {
    /// Same as [`crate::cli::Cli::dir`].
    pub dir: PathBuf,

    /// Same as [`crate::cli::Cli::filename`].
    pub filename: String,

    /// Same as [`crate::cli::Cli::backup_dir`].
    pub backup_dir: PathBuf,

    /// Same as [`crate::cli::Cli::always_skip`].
    pub always_skip: bool,

    /// Same as [`crate::cli::Cli::always_backup`].
    pub always_backup: bool,
}

impl Params {
    /// Creates parameters for the app, i.e. an aggregation of user-defined
    /// arguments/configurations, with verification of the validity of those
    /// arguments/configurations.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use clap::Parser;
    /// use mksls::cfg::Config;
    /// use mksls::cli::Cli;
    /// use mksls::params::Params;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cli = Cli::parse();
    /// let cfg: Config = confy::load("my_crate", "config")?;
    ///
    /// let params = Params::new(cli, cfg)?;
    /// println!("{:?}", params);
    /// # Ok(())
    /// # }
    /// ```
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

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestCase {
        cli: Cli,
        cfg: Config,
        params: Params,
    }

    #[test]
    fn cli_takes_precedence_on_config() {
        let test_cases = vec![
            TestCase {
                // Cli takes precedence
                cli: Cli {
                    dir: PathBuf::from("dir"),
                    filename: Some(String::from("cli_filename")),
                    backup_dir: Some(PathBuf::from("/cli/backup/dir")),
                    always_skip: false,
                    always_backup: true,
                },
                cfg: Config {
                    filename: String::from("cfg_filename"),
                    backup_dir: PathBuf::from("/cfg/backup/dir"),
                    always_skip: true,
                    always_backup: false,
                },
                params: Params {
                    dir: PathBuf::from("dir"),
                    filename: String::from("cli_filename"),
                    backup_dir: PathBuf::from("/cli/backup/dir"),
                    always_skip: false,
                    always_backup: true,
                },
            },
            // When option not defined via Cli, backup to Config
            TestCase {
                cli: Cli {
                    dir: PathBuf::from("dir"),
                    filename: None,
                    backup_dir: None,
                    always_skip: false,
                    always_backup: false,
                },
                cfg: Config {
                    filename: String::from("cfg_filename"),
                    backup_dir: PathBuf::from("/cfg/backup/dir"),
                    always_skip: true,
                    always_backup: false,
                },
                params: Params {
                    dir: PathBuf::from("dir"),
                    filename: String::from("cfg_filename"),
                    backup_dir: PathBuf::from("/cfg/backup/dir"),
                    always_skip: true,
                    always_backup: false,
                },
            },
            // A mix of options coming from Cli and others from Config
            TestCase {
                cli: Cli {
                    dir: PathBuf::from("dir"),
                    filename: Some(String::from("cli_filename")),
                    backup_dir: None,
                    always_skip: false,
                    always_backup: false,
                },
                cfg: Config {
                    filename: String::from("cfg_filename"),
                    backup_dir: PathBuf::from("/cfg/backup/dir"),
                    always_skip: true,
                    always_backup: false,
                },
                params: Params {
                    dir: PathBuf::from("dir"),
                    filename: String::from("cli_filename"),
                    backup_dir: PathBuf::from("/cfg/backup/dir"),
                    always_skip: true,
                    always_backup: false,
                },
            },
        ];

        for test_case in test_cases {
            let params = Params::new(test_case.cli, test_case.cfg).expect(
                "Params::new should have succeed. There must be an error in the test case.",
            );
            assert_eq!(
                params, test_case.params,
                "Expected {:?}, but got {:?}",
                test_case.params, params
            );
        }
    }
}
