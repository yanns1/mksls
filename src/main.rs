use clap::{crate_name, Parser};
use mksls::cfg::Config;
use mksls::cli::Cli;
use mksls::dir::error::{DirCreationFailed, DirDoesNotExist};
use mksls::engine::Engine;
use mksls::params::Params;
use std::fs;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cfg: Config = confy::load(crate_name!(), crate_name!())?;

    let params = Params::new(cli, cfg)?;
    if !params.dir.is_dir() {
        Err(DirDoesNotExist(params.dir.clone()))?;
    }
    if !params.backup_dir.is_dir() {
        if let Err(err) = fs::create_dir_all(params.backup_dir.as_path()) {
            Err(DirCreationFailed(params.backup_dir.clone(), err))?;
        }
    }

    Engine::new(params).run()
}
