use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "Templar", about = "A templating engine for config files")]
pub(super) struct Opt {
    /// Activate debug mode
    #[structopt(short, long)]
    pub debug: bool,

    /// Templar Command to execute
    #[structopt(subcommand)]
    pub command: Option<TemplarCommand>,
}

pub(super) fn from_args() -> Opt {
    Opt::from_args()
}

#[derive(Debug, StructOpt)]
pub enum TemplarCommand {
    /// Run templar
    Run(Run),
    /// Generate the lua module for Templar
    Generate(Generate),
}

#[derive(Debug, StructOpt)]
pub struct Generate {
    /// Path to the file to generate
    #[structopt(short, long)]
    pub file_path: Option<PathBuf>,
}

#[derive(Debug, StructOpt)]
pub struct Run {
    /// Path to the file to generate
    #[structopt(short, long)]
    pub config_path: Option<PathBuf>,
}
