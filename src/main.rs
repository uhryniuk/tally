use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use dirs::home_dir;
use prettytable::{row, Table};
use tally_cli::models::Counter;
use tally_cli::{database, template};
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

const DATABASE_FILE: &str = "tally.db";
const DATA_DIR: &str = ".tally";

fn main() -> Result<()> {
    let app = Command::new("tally")
        .version(env!("CARGO_PKG_VERSION"))
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
        .subcommand(Command::new("delete").about("Delete a given counter"))
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
        .subcommand(
            Command::new("nuke").about("Nuke the counter database").arg(
                Arg::new("yes")
                    .long("yes")
                    .short('y')
                    .required(false)
                    .action(clap::ArgAction::SetTrue)
                    .help("Skip the confirmation prompt"),
            ),
        );

    let matches = app.get_matches();

    // Create the ~/.tally directory and conn file
    let home: PathBuf = home_dir().expect("Couldn't get $HOME directory");
    let data_dir = home.join(PathBuf::from(DATA_DIR));
    std::fs::create_dir_all(data_dir.clone())?;

    // Create path to database file and initialize
    let database_path = data_dir.join(PathBuf::from(DATABASE_FILE));
    let conn = database::Connection::new(&database_path.clone().to_string_lossy())
        .expect("Cannot connect to database.");

    let name: String = match matches.get_one::<String>("name") {
        Some(n) => n.clone(),
        None => Counter::get_default(conn.get())?
            .ok_or_else(|| anyhow!("no default counter set; run 'tally <name> set --default'"))?,
    };

    let mut counter = match Counter::get(conn.get(), &name)? {
        Some(c) => c,
        None => {
            let c = Counter::new(&name);
            c.insert(conn.get())?;
            c
        }
    };

    let is_quiet = matches.get_one::<bool>("quiet").cloned().unwrap();
    let is_raw = matches.get_flag("raw");

    // divert logic to subcommand
    match matches.subcommand() {
        Some(("set", sub_mat)) => {
            if let Some(count) = sub_mat.get_one::<String>("count").cloned() {
                match count.parse::<i64>() {
                    Ok(count) => counter.count = count,
                    Err(e) => {
                        eprintln!(
                            "failed to set 'count' for counter '{}' because: {}",
                            counter.name, e
                        );
                        exit(1);
                    }
                }
            }

            if let Some(step) = sub_mat.get_one::<String>("step").cloned() {
                match step.parse::<i64>() {
                    Ok(step) => counter.step = step,
                    Err(e) => {
                        eprintln!(
                            "failed to set 'step' for counter '{}' because: {}",
                            counter.name, e
                        );
                        exit(1);
                    }
                }
            }

            if let Some(template) = sub_mat.get_one::<String>("template").cloned() {
                counter.template = template
            }
            if sub_mat.get_flag("default") {
                counter.set_default(conn.get())?;
            }
            counter.update(conn.get())?;
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
                .padding(0, 2)
                .build();
            table.set_format(format);
            table.add_row(row!["Name", "Count", "Step", "Template", "Default"]);

            if sub_mat.get_flag("no-headers") {
                table.remove_row(0);
            }

            // Add rows of data to table
            let rows = Counter::get_all(conn.get())?;
            let default = Counter::get_default(conn.get())?.unwrap();
            for row in rows.iter() {
                let is_default = if default == row.name { "*" } else { "" };
                table.add_row(row![
                    row.name,
                    row.count,
                    row.step,
                    row.template,
                    is_default
                ]);
            }
            table.printstd();
        }
        Some(("nuke", sub_mat)) => {
            let confirmed = if sub_mat.get_flag("yes") {
                true
            } else {
                print!("Are you sure wish to nuke? (y/n): ");
                std::io::stdout().flush().unwrap();

                let mut input = String::new();
                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");

                input.to_lowercase().trim() == "y"
            };

            if confirmed {
                std::fs::remove_file(database_path)?;
                println!("Database deleted successfully.");
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

    Ok(())
}
