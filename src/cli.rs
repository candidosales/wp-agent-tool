use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Print the report as JSON on stdout (progress goes to stderr)
    #[arg(long)]
    pub json: bool,
}
