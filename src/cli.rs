use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,
    #[clap(long("device"), short, value_name("DEVICES"), num_args(1..))]
    pub devices: Option<Vec<String>>,
}

impl Cli {
    pub fn command(self) -> Commands {
        self.command.unwrap_or_default()
    }
}

#[derive(Subcommand, Debug, Default)]
pub enum Commands {
    Layout(Layout),
    Devices,
    #[default]
    /// [Default] run swayboard service
    Run,
}

#[derive(Parser, Debug)]
pub struct Layout {
    #[clap(subcommand)]
    pub(crate) command: LayoutSubcommands,
}

// #[derive(Debug, Parser)]
// pub struct LayoutArg {
// Name(String),
// Id(usize),
// }

#[derive(Parser, Debug)]
pub(crate) struct SetLayoutArgs {
    pub(crate) layout: String,
}

#[derive(Subcommand, Debug, Default)]
pub(crate) enum LayoutSubcommands {
    Get,
    Set(SetLayoutArgs),
    #[default]
    List,
}
