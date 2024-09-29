use anyhow::Result;
//use bore_cli::{client::Client, server::Server};
use clap::{error::ErrorKind, Arg, Command, CommandFactory, Parser, Subcommand};

#[tokio::main]
async fn run(command: Command) -> Result<()> {
    //match command {
    //    Command::Set {
    //        local_host,
    //        local_port,
    //        to,
    //        port,
    //        secret,
    //    } => {
    //        //let client = Client::new(&local_host, local_port, &to, port, secret.as_deref()).await?;
    //        //client.listen().await?;
    //    }
    //    Command::Server {
    //        min_port,
    //        max_port,
    //        secret,
    //    } => {
    //        let port_range = min_port..=max_port;
    //        if port_range.is_empty() {
    //            Args::command()
    //                .error(ErrorKind::InvalidValue, "port range is empty")
    //                .exit();
    //        }
    //        Server::new(port_range, secret.as_deref()).listen().await?;
    //    }
    //}
    //
    println!("Writing from run");

    Ok(())
}

#[tokio::main]
async fn runner(command: Command) -> Result<()> {
    //match command {
    //    Command::Set {
    //        local_host,
    //        local_port,
    //        to,
    //        port,
    //        secret,
    //    } => {
    //        //let client = Client::new(&local_host, local_port, &to, port, secret.as_deref()).await?;
    //        //client.listen().await?;
    //    }
    //    Command::Server {
    //        min_port,
    //        max_port,
    //        secret,
    //    } => {
    //        let port_range = min_port..=max_port;
    //        if port_range.is_empty() {
    //            Args::command()
    //                .error(ErrorKind::InvalidValue, "port range is empty")
    //                .exit();
    //        }
    //        //Server::new(port_range, secret.as_deref()).listen().await?;
    //    }
    //}

    println!("Writing from RUNNER");

    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    //let _ = run(Args::parse().command);
    //runner(Args::parse().command)
    let app = Command::new("tally")
        .about("A global counter")
        .arg(
            Arg::new("name")
                .required(false)
                .index(1)
                .help("Name of counter"),
        )
        .arg(
            Arg::new("raw")
                .required(false)
                .long("raw")
                .help("Render counter without template (if template is set)"),
        )
        .subcommand(
            Command::new("set")
                .about("Set configuration of the provided counter")
                .arg(
                    Arg::new("count")
                        .required(false)
                        .index(1)
                        .help("Set the count to this integer"),
                )
                .arg(
                    Arg::new("inc")
                        .required(false)
                        .long("inc")
                        .short('i')
                        .help("Default increment amount for given counter"),
                )
                .arg(
                    Arg::new("dec")
                        .required(false)
                        .long("dec")
                        .short('d')
                        .help("Default decrement amount for given counter"),
                )
                .arg(
                    Arg::new("template")
                        .required(false)
                        .long("template")
                        .short('t')
                        .help("Template to render when counter is invoked"),
                )
                .arg(
                    Arg::new("default")
                        .required(false)
                        .long("default")
                        .help("Provided counter becomes the default"),
                ),
        )
        .subcommand(
            Command::new("inc").about("Increment a given counter").arg(
                Arg::new("amount")
                    .required(false)
                    .index(1)
                    .help("Amount to increment the counter by"),
            ),
        )
        .subcommand(
            Command::new("dec").about("Decrement a given counter").arg(
                Arg::new("amount")
                    .required(false)
                    .index(1)
                    .help("Amount to decrement the counter by"),
            ),
        )
        .subcommand(
            Command::new("delete").about("Dlete a given counter").arg(
                Arg::new("counter")
                    .required(false)
                    .index(1)
                    .help("Counter to delete"),
            ),
        )
        .subcommand(Command::new("list").about("List all of the active counters"));

    let matches = app.get_matches();

    match matches.subcommand() {
        Some(("set", sub_mat)) => {
            println!("Running set {:?}", sub_mat);
        }
        Some(("inc", sub_mat)) => {
            println!("Running inc {:?}", sub_mat);
        }
        Some(("dec", sub_mat)) => {
            println!("Running dec {:?}", sub_mat);
        }
        Some(("delete", sub_mat)) => {
            println!("Running delete {:?}", sub_mat);
        }
        Some(("list", sub_mat)) => {
            println!("Running list {:?}", sub_mat);
        }
        None => {
            println!("No subcommand provided");
        }
        _ => {
            eprintln!("Weird context error");
            std::process::exit(1);
        }
    }

    Ok(())
}
