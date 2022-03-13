#![allow(dead_code)]
#![feature(drain_filter)]

use anyhow::Context;

mod commands;
mod conductor;
mod config;
mod opt;
mod paths;
mod utils;

fn main() {
    let _templar_paths = paths::TemplarPaths::try_from_env().unwrap();
    let opt = opt::from_args();

    if let Some(command) = opt.command {
        match command {
            opt::TemplarCommand::Run => commands::run(),
            opt::TemplarCommand::Generate => commands::generate(),
        }
        .with_context(|| format!("Failed to execute command: {:?}", command))
        .unwrap();
    } else {
        println!("No command specified");
    }
}
