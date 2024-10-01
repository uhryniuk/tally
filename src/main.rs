mod database;

use anyhow::Result;
use clap::{Arg, Command};
use prettytable::{row, Table};
use std::fs::create_dir;
use std::path::PathBuf;

const DATABASE_FILE: &str = "tally.db";
const DATA_DIR: &str = ".tally";

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
                    // TODO are the inc and dec not the same step value?
                    // Consider this further...
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
            Command::new("delete").about("Delete a given counter").arg(
                Arg::new("counter")
                    .required(false)
                    .index(1)
                    .help("Counter to delete"),
            ),
        )
        .subcommand(Command::new("list").about("List all of the active counters"));

    let matches = app.get_matches();

    // Create the ~/.tally directory db file
    let home: PathBuf = std::env::home_dir().expect("Couldn't get $HOME directory");
    let data_dir = home.join(PathBuf::from(DATA_DIR));
    std::fs::create_dir_all(data_dir.clone())?;

    // Create path to database file and initialize
    let database_path = data_dir.join(PathBuf::from(DATABASE_FILE));
    let db = database::Database::new(&database_path.to_string_lossy())
        .expect("Cannot connect to database.");

    // parse top level args
    let default_name = db.get_default_counter()?;
    let name = matches
        .get_one::<String>("name")
        .cloned()
        .unwrap_or_else(|| default_name.clone());

    let is_raw = matches.get_flag("raw");
    let print_count = |count: i64| {
        if is_raw {
            println!("{}", count);
        } else {
            // TODO add render template call
            println!("{}", count)
        }
    };

    // divert logic to subcommand
    match matches.subcommand() {
        Some(("set", sub_mat)) => {
            println!("Running set {:?}", sub_mat);
        } // TODO should we change inc/dec -> add/sub?
        // Easier to add mul/div later too
        Some(("inc", sub_mat)) => {
            let mut amount: i64 = db.get_step(&name)?;
            if let Some(given) = sub_mat.get_one::<String>("amount").cloned() {
                amount = given.parse()?;
            }
            let count = db.increment_and_get_count(&name, amount)?;
            print_count(count);
        }
        Some(("dec", sub_mat)) => {
            let mut amount: i64 = db.get_step(&name)?;
            if let Some(given) = sub_mat.get_one::<String>("amount").cloned() {
                amount = given.parse()?;
            }

            let count = db.decrement_and_get_count(&name, amount)?;
            print_count(count);
        }
        Some(("delete", _sub_mat)) => db.delete_counter(&name)?,
        Some(("list", _sub_mat)) => {
            // Create and format table
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .column_separator(' ') // No column separators
                .borders(' ') // No borders around the table
                .padding(0, 1) // No padding
                .build();
            table.set_format(format);
            table.add_row(row!["Name", "Count", "Step", "Template", "Default"]);

            // Add rows of data to table
            let rows = db.get_all_counters()?;
            for row in rows.iter() {
                table.add_row(row![
                    row.name,
                    row.count,
                    row.step,
                    row.template, // TODO render the template here, or replace with None
                    row.is_default
                ]);
            }
            table.printstd();
        }
        None => print_count(db.get_count(&name)?),
        _ => {
            eprintln!("Weird context error");
            std::process::exit(1);
        }
    }

    Ok(())
}
