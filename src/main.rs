mod database;
mod models;
mod template;

use anyhow::Result;
use clap::{Arg, Command};
use dirs::home_dir;
use models::Counter;
use prettytable::{row, Table};
use std::io::Write;
use std::path::PathBuf;

const DATABASE_FILE: &str = "tally.db";
const DATA_DIR: &str = ".tally";

fn main() -> Result<()> {
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
        .arg(
            Arg::new("quiet")
                .required(false)
                .long("quiet")
                .short('q')
                .action(clap::ArgAction::SetTrue)
                .help("Add to counter but don't write to stdout."),
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
                    Arg::new("step")
                        .required(false)
                        .long("step")
                        .short('s')
                        .help("Default increment amount for given counter"),
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
                        .action(clap::ArgAction::SetTrue)
                        .help("Provided counter becomes the default"),
                ),
        )
        .subcommand(
            Command::new("add").about("Increment a given counter").arg(
                Arg::new("amount")
                    .required(false)
                    .index(1)
                    .help("Amount to increment the counter by"),
            ),
        )
        .subcommand(
            Command::new("sub").about("Decrement a given counter").arg(
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
        .subcommand(
            Command::new("list")
                .about("List all of the active counters")
                .arg(
                    Arg::new("no-headers")
                        .long("no-headers")
                        .required(false)
                        .action(clap::ArgAction::SetTrue)
                        .help("List counters without column headers"),
                ),
        )
        .subcommand(Command::new("nuke").about("Nuke the counter database"));

    let matches = app.get_matches();

    // Create the ~/.tally directory and conn file
    let home: PathBuf = home_dir().expect("Couldn't get $HOME directory");
    let data_dir = home.join(PathBuf::from(DATA_DIR));
    std::fs::create_dir_all(data_dir.clone())?;

    // Create path to database file and initialize
    let database_path = data_dir.join(PathBuf::from(DATABASE_FILE));
    let conn = database::Connection::new(&database_path.clone().to_string_lossy())
        .expect("Cannot connect to database.");

    // parse top level args
    // let default_name = conn.get_default_counter()?;
    // TODO impl the new default table
    let name = match matches.get_one::<String>("name") {
        Some(n) => n,
        None => &Counter::get_default(conn.get())?.unwrap(),
    };

    let mut counter = match Counter::get(conn.get(), name)? {
        Some(c) => c,
        None => {
            let c = Counter::new(name);
            c.insert(conn.get())?;
            c
        }
    };

    let is_quiet = matches.get_one::<bool>("quiet").cloned().unwrap();
    let is_raw = matches.get_flag("raw");

    // divert logic to subcommand
    match matches.subcommand() {
        Some(("set", sub_mat)) => {
            if let Some(count) = sub_mat.get_one::<i64>("count").cloned() {
                counter.count = count;
            }

            if let Some(step) = sub_mat.get_one::<i64>("step").cloned() {
                counter.step = step;
            }

            if let Some(template) = sub_mat.get_one::<String>("template").cloned() {
                counter.template = template
            }
            if let Some(_) = sub_mat.get_one::<bool>("default").cloned() {
                counter.set_default(conn.get())?;
            }
        }
        Some(("add", sub_mat)) => {
            let amount = match sub_mat.get_one::<String>("amount") {
                Some(amount) => amount.parse::<i64>()?,
                None => counter.step,
            };

            counter.count += amount;
            counter.update(conn.get())?;

            if !is_quiet {
                if is_raw {
                    println!("{}", counter.count);
                } else {
                    println!("{}", template::render(&conn, &counter.name)?);
                }
            }
        }
        Some(("sub", sub_mat)) => {
            let amount = match sub_mat.get_one::<String>("amount") {
                Some(amount) => amount.parse::<i64>()?,
                None => counter.step,
            };

            counter.count -= amount;
            counter.update(conn.get())?;

            if !is_quiet {
                if is_raw {
                    println!("{}", counter.count);
                } else {
                    println!("{}", template::render(&conn, &counter.name)?);
                }
            }
        }
        Some(("delete", _sub_mat)) => Counter::delete(conn.get(), &counter.name)?,
        Some(("list", sub_mat)) => {
            // Create and format table
            let mut table = Table::new();
            let format = prettytable::format::FormatBuilder::new()
                .column_separator(' ')
                .borders(' ')
                .padding(0, 1)
                .build();
            table.set_format(format);
            table.add_row(row!["Name", "Count", "Step", "Template", "Default"]);

            let has_headers = sub_mat.get_one::<bool>("no-headers").cloned().unwrap();
            if has_headers {
                table.remove_row(0);
            }

            // Add rows of data to table
            let rows = Counter::get_all(conn.get())?;
            let default = Counter::get_default(conn.get())?.unwrap();
            for row in rows.iter() {
                let is_default = if default == row.name { "*" } else { "" };
                table.add_row(row![row.name, row.count, row.step, row.template, is_default]);
            }
            table.printstd();
        }
        Some(("nuke", _sub_mat)) => {
            print!("Are you sure wish to nuke? (y/n): ");
            std::io::stdout().flush().unwrap();

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            if input.to_lowercase().trim() == "y" {
                println!("Database deleted successfully.");
                std::fs::remove_file(database_path)?;
            }
        }
        None => {
            if !is_quiet {
                if is_raw {
                    println!("{}", counter.count);
                } else {
                    println!("{}", template::render(&conn, &counter.name)?);
                }
            }
        }
        _ => {
            eprintln!("Unable to handle input");
            std::process::exit(1);
        }
    }

    counter.update(conn.get())?;

    Ok(())
}
