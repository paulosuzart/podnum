use clap::Parser;

#[derive(Parser)]
#[command(author="Paulo Suzart", version, about ="Assign integer numbers to pod hosts")]
pub(crate) struct PodNumArgs {
    #[arg(short, long)]
    pub(crate) pid: u64,
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ',', required = true)]
    pub(crate) nodes: Vec<String>,
    #[arg(short = 'x', long, value_parser, num_args = 1.., value_delimiter = ',', required = true)]
    pub(crate) pids: Vec<u64>,
}