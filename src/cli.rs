use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
    #[clap(long("device"), short, value_name("DEVICE"), multiple_values(true))]
    pub devices: Option<Vec<String>>,
}

#[derive(Subcommand, Debug, Default)]
pub enum Commands {
    Layout(Layout),
    Devices,
    #[default]
    Run,
}

#[derive(Parser, Debug)]
pub struct Layout {
    #[clap(subcommand)]
    pub command: LayoutSubcommands,
}

// #[derive(Debug, Parser)]
// pub struct LayoutArg {
    // Name(String),
    // Id(usize),
// }

#[derive(Parser, Debug)]
pub struct SetLayoutArgs {
    pub layout: String,
}

#[derive(Subcommand, Debug, Default)]
pub enum LayoutSubcommands {
    Get,
    Set(SetLayoutArgs),
    #[default]
    List
}
