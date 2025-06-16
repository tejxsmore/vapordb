use clap::{Parser, Subcommand};

mod utils;
mod commands {
    pub mod string;
    pub mod hash;
    pub mod list;
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
    // String (Key-Value)
    Set { key: String, value: String },
    Get { key: String },
    Del { key: String },
    SetExpiring {
        key: String,
        value: String,
        #[arg(short, long)]
        ttl: u64,
    },

    // Server
    Start,

    // Hash
    HSet { key: String, field: String, value: String },
    HGet { key: String, field: String },
    HDel { key: String, field: String },

    // List
    LPush { key: String, value: String },
    RPush { key: String, value: String },
    LPop { key: String },
    RPop { key: String },
    LRange { key: String, start: usize, end: usize },

    // Set
    SAdd { key: String, value: String },
    SRem { key: String, value: String },
    SMembers { key: String },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // String commands
        Commands::Set { key, value } => {
            commands::string::handle_set(&key, &value);
        }
        Commands::Get { key } => {
            commands::string::handle_get(&key);
        }
        Commands::Del { key } => {
            commands::string::handle_del(&key);
        }
        Commands::SetExpiring { key, value, ttl } => {
            commands::string::handle_set_expiring(&key, &value, ttl);
        }

        // Server
        Commands::Start => {
            commands::start::start_server();
        }

        // Hash commands
        Commands::HSet { key, field, value } => {
            commands::hash::handle_hset(&key, &field, &value);
        }
        Commands::HGet { key, field } => {
            commands::hash::handle_hget(&key, &field);
        }
        Commands::HDel { key, field } => {
            commands::hash::handle_hdel(&key, &field);
        }

        // List commands
        Commands::LPush { key, value } => {
            commands::list::handle_lpush(&key, &value);
        }
        Commands::RPush { key, value } => {
            commands::list::handle_rpush(&key, &value);
        }
        Commands::LPop { key } => {
            commands::list::handle_lpop(&key);
        }
        Commands::RPop { key } => {
            commands::list::handle_rpop(&key);
        }
        Commands::LRange { key, start, end } => {
            commands::list::handle_lrange(&key, start, end);
        }

        // Set commands
        Commands::SAdd { key, value } => {
            commands::set::handle_sadd(&key, &value);
        }
        Commands::SRem { key, value } => {
            commands::set::handle_srem(&key, &value);
        }
        Commands::SMembers { key } => {
            commands::set::handle_smembers(&key);
        }
    }
}
