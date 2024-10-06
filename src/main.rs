mod database;
mod template;

use anyhow::Result;
use clap::{Arg, Command};
use prettytable::{row, Table};
use std::io::{self, Write};
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
            Command::new("add")
                .about("Increment a given counter")
                .arg(
                    Arg::new("amount")
                        .required(false)
                        .index(1)
                        .help("Amount to increment the counter by"),
                )
                .arg(
                    Arg::new("quiet")
                        .required(false)
                        .long("quiet")
                        .short('q')
                        .action(clap::ArgAction::SetTrue)
                        .help("Add to counter but don't write to stdout."),
                ),
        )
        .subcommand(
            Command::new("sub")
                .about("Decrement a given counter")
                .arg(
                    Arg::new("amount")
                        .required(false)
                        .index(1)
                        .help("Amount to decrement the counter by"),
                )
                .arg(
                    Arg::new("quiet")
                        .required(false)
                        .long("quiet")
                        .short('q')
                        .action(clap::ArgAction::SetTrue)
                        .help("Subtract from counter but don't write to stdout."),
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

    // Create the ~/.tally directory and db file
    let home: PathBuf = std::env::home_dir().expect("Couldn't get $HOME directory");
    let data_dir = home.join(PathBuf::from(DATA_DIR));
    std::fs::create_dir_all(data_dir.clone())?;

    // Create path to database file and initialize
    let database_path = data_dir.join(PathBuf::from(DATABASE_FILE));
    let db = database::Database::new(&database_path.clone().to_string_lossy())
        .expect("Cannot connect to database.");

    // parse top level args
    let default_name = db.get_default_counter()?;
    let name = matches
        .get_one::<String>("name")
        .cloned()
        .unwrap_or_else(|| default_name.clone());

    let is_raw = matches.get_flag("raw");
    let print_count = || -> Result<()> {
        if is_raw {
            println!("{}", db.get_count(&name)?);
        } else {
            println!("{}", template::render(&db, &name)?);
        }
        Ok(())
    };

    // divert logic to subcommand
    match matches.subcommand() {
        Some(("set", sub_mat)) => {
            if let Some(count) = sub_mat.get_one::<String>("count").cloned() {
                db.set_count(&name, count.parse()?)?;
            }

            if let Some(step) = sub_mat.get_one::<String>("step").cloned() {
                db.set_step(&name, step.parse()?)?;
            }

            if let Some(template) = sub_mat.get_one::<String>("template").cloned() {
                db.set_template(&name, &template)?;
            }

            if let Some(default) = sub_mat.get_one::<bool>("default").cloned() {
                if default {
                    db.set_default(&name, default)?;
                }
            }
        }
        Some(("add", sub_mat)) => {
            db.init_counter(&name)?;
            let mut amount: i64 = db.get_step(&name)?;
            if let Some(given) = sub_mat.get_one::<String>("amount").cloned() {
                amount = given.parse()?;
            }

            let _count = db.increment_and_get_count(&name, amount)?;

            let is_quiet = sub_mat.get_one::<bool>("quiet").cloned().unwrap();
            if !is_quiet {
                print_count()?;
            }
        }
        Some(("sub", sub_mat)) => {
            db.init_counter(&name)?;
            let mut amount: i64 = db.get_step(&name)?;
            if let Some(given) = sub_mat.get_one::<String>("amount").cloned() {
                amount = given.parse()?;
            }

            let _count = db.decrement_and_get_count(&name, amount)?;

            let is_quiet = sub_mat.get_one::<bool>("quiet").cloned().unwrap();
            if !is_quiet {
                print_count()?;
            }
        }
        Some(("delete", _sub_mat)) => db.delete_counter(&name)?,
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
            let rows = db.get_all_counters()?;
            for row in rows.iter() {
                table.add_row(row![
                    row.name,
                    row.count,
                    row.step,
                    row.template,
                    row.is_default
                ]);
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
            db.init_counter(&name)?;
            db.get_count(&name)?;
            print_count()?;
        }
        _ => {
            eprintln!("Unable to hanlde input");
            std::process::exit(1);
        }
    }

    Ok(())
}
