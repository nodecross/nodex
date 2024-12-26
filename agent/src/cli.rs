use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Default)]
pub struct AgentOptions {
    #[arg(long, help = "Enable configuration")]
    pub config: bool,

    #[command(subcommand)]
    pub command: Option<AgentCommands>,
}

#[derive(Subcommand, Debug)]
pub enum AgentCommands {
    #[command(about = "help for DID")]
    Did,
    #[command(about = "help for Network")]
    Network {
        #[command(subcommand)]
        command: NetworkSubCommands,
    },
}

#[derive(Subcommand, Debug)]
pub enum NetworkSubCommands {
    #[command(about = "Set a network configuration")]
    Set {
        #[arg(short, long)]
        key: String,
        #[arg(short, long)]
        value: String,
    },
    #[command(about = "Get a network configuration")]
    Get {
        #[arg(short, long)]
        key: String,
    },
}
