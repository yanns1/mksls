pub mod cfg;
pub mod cli;
pub mod dir;
pub mod engine;
pub mod line;
pub mod params;

use crate::cfg::Config;
use crate::cli::Cli;
use crate::params::Params;
use clap::{crate_name, Parser};
use engine::Engine;
use std::fs;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg: Config = confy::load(crate_name!(), crate_name!())?;

    let params = Params::new(cli, cfg)?;
    if !params.dir.is_dir() {
        Err(dir::error::DirDoesNotExist(params.dir.clone()))?;
    }
    if !params.backup_dir.is_dir() {
        if let Err(err) = fs::create_dir_all(params.backup_dir.as_path()) {
            Err(dir::error::DirCreationFailed(
                params.backup_dir.clone(),
                err,
            ))?;
        }
    }

    Engine::new(params).run()
}
