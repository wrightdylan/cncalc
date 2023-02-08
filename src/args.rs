use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "Star Cluster Neighbour Calculator")]
#[command(author, version, about, long_about = None)]
#[command(help_template = "\
{before-help}{name} {version}
{author-with-newline}{about-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
")]
pub struct CalcArgs {
    /// Input filename
    pub filename: String
}