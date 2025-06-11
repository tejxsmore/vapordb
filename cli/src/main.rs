use clap::{Parser, Subcommand}; // ✅ Fix: Import Subcommand derive macro

mod utils;
mod commands {
    pub mod get;
    pub mod set;
    pub mod start;
}

#[derive(Parser)]
#[command(name = "vapordb-cli", version = "0.1", about = "VaporDB CLI Client")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Set {
        key: String,
        value: String,
    },
    Get {
        key: String,
    },
    Del {
        key: String,
    },
    SetExpiring {
        key: String,
        value: String,
        #[arg(short, long)]
        ttl: u64,
    },
    Start,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Set { key, value } => {
            commands::set::handle_set(&key, &value);
        }
        Commands::Get { key } => {
            commands::get::handle_get(&key);
        }
        Commands::Del { key } => {
            commands::set::handle_del(&key);
        }
        Commands::SetExpiring { key, value, ttl } => {
            commands::set::handle_set_expiring(&key, &value, ttl);
        }
        Commands::Start => {
            commands::start::start_server();
        }
    }
}
