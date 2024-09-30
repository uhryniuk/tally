mod database;

use anyhow::Result;
use clap::{error::ErrorKind, Arg, Command, CommandFactory, Parser, Subcommand};
use sqlite::Connection;

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
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

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
                .action(clap::ArgAction::SetTrue)
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

    // Parse config file (contains database location, default counter name) - Future
    // Check if data dir exists, if not create
    // Check if database file exists, if not create, seed database
    // Get database connection
    // Pass connection and context variables to fn that handle each subcommand
    let db = database::Database::new("database.db").expect("Couldn't connect to database");

    // Check if a specific table was supplied.
    // Otherwise get the default table from the database.

    let name = matches
        .get_one::<String>("name")
        .cloned()
        .unwrap_or_else(|| String::from("tally"));

    let is_raw = matches.get_flag("raw");

    match matches.subcommand() {
        Some(("set", sub_mat)) => {
            println!("Running set {:?}", sub_mat);
        }
        Some(("inc", sub_mat)) => {
            let amount = 1; // TODO remove hardcoded amount to increment by.
            let count = db.increment_and_get_count(&name, amount)?;
            println!("{}", count);
        }
        Some(("dec", sub_mat)) => {
            let amount = 1; // TODO remove hardcoded amount to increment by.
            let count = db.decrement_and_get_count(&name, amount)?;
            println!("{}", count);
        }
        Some(("delete", sub_mat)) => {
            println!("Running delete {:?}", sub_mat);
        }
        Some(("list", sub_mat)) => {
            println!("Running list {:?}", sub_mat);
        }
        None => {
            let count = db.get_count(&name)?;

            match is_raw {
                true => println!("{}", count),
                false => println!("{}", count), // TODO add render template call
            }
            // attempt to get count
            // if no row returns, create a row and return 0
            // print out the value to stdout.
        }
        _ => {
            eprintln!("Weird context error");
            std::process::exit(1);
        }
    }

    Ok(())
}
