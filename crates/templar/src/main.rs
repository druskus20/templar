#![allow(dead_code)]
#![feature(drain_filter)]

use std::{env::current_dir, path::PathBuf};

use anyhow::{Context, Result};

mod commands;
mod config;
mod opt;
mod paths;
mod templating;
mod utils;

fn main() {
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
