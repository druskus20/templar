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
    Run,
    /// Generate the lua module for Templar
    Generate,
}
