use clap::Parser;

#[derive(Parser, Debug)]
#[command(author="Paulo Suzart", version, about ="Assign integer numbers to pod hosts")]
pub(crate) struct PodNumArgs {
    // Pid of podnum instance
    #[arg(short, long)]
    pub(crate) pid: u64,
    // Node pids
    #[arg(short, long, value_parser, num_args = 1.., value_delimiter = ',')]
    pub(crate) nodes: Vec<u64>,
}