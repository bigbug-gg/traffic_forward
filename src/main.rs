use clap::{Parser, Subcommand};
use traffic_forward::{iptables::tools::is_root, service};

/// Quickly set up traffic forwarding
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli{
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand,Debug, Clone)]
enum Commands {
    /// Add forward
    #[command(arg_required_else_help = true)]
    Add {
        /// The target IP, Format x.x.x.x:xxxx
        target: String,

        /// The port of local.
        port: String,
    },

    /// Delete all target rule for forward.
    #[command(arg_required_else_help = true)]
    Delete {
        ip: String
    },

    /// Query the traffic used. (Unit: byte)
    #[command(arg_required_else_help = true)]
    Query {
        ip: String 
    },

    /// List forward info
    List,

    /// Web API switch, start or off
    Web
}

 fn main()  {
   
    if !is_root() {
        println!("Root permission required, operation cancelled");
        return;
    }


    let args = Cli::parse();
    match args.command {

        Commands::Add { target, port } => {
            let target: Vec<&str> = target.split(':').collect();
            if target.len() != 2 {
                println!("Target Port Error");
                return;
            }

            let is_ok = service::add(  
                target[0],
                target[1],
                &port
            );

            if let Err(e) =  is_ok{
                println!("Add  error: {e}");
                return;
            }

            println!("Add completed");
            return;
        },

        Commands::Delete { ip } => {
            let is_ok = service::del(&ip);
            if let Err(e) =  is_ok{
                println!("Delete error: {e}");
                return;
            }

            println!("Delete completed");
            return;
        },
        
        Commands::Query { ip } => {
            let is_ok = service::traffic(&ip);
            if let Err(e) =  is_ok{
                println!("Query error: {e}");
                return;
            }

            println!("Delete completed");
            return;
        },
        
        Commands::Web => {
            traffic_forward::api_server();
        },
        
        Commands::List => {
            let is_ok = service::list();
            if let Some(e) =  is_ok{
                println!("{}", e);
                return;
            }

            println!("No Data");
            return;
        },
    }

}