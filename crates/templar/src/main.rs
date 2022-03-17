#![allow(dead_code)]
#![feature(drain_filter)]

use anyhow::Context;

mod commands;
mod conductor;
mod config;
mod opt;
mod utils;

fn main() {
    let opt = opt::from_args();

    if let Some(command) = opt.command {
        match &command {
            opt::TemplarCommand::Run(x) => commands::run(x),
            opt::TemplarCommand::Generate(x) => commands::generate(x),
        }
        .with_context(|| format!("Failed to execute command: {:?}", command))
        .unwrap();
    } else {
        println!("No command specified");
    }
}
