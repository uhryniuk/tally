use anyhow::Result;
use sqlite::ConnectionThreadSafe;

pub struct Database {
    pub conn: ConnectionThreadSafe,
}

impl Database {
    pub fn new(name: &str) -> Result<Database> {
        let db = Database {
            conn: sqlite::Connection::open_thread_safe(name)?,
        };
        db.init_database()?;
        db.get_default_counter()?;
        Ok(db)
    }

    fn init_database(&self) -> Result<()> {
        // create the default table
        self.conn.execute(
            "
            CREATE TABLE IF NOT EXISTS counters (
                name TEXT NOT NULL,
                count INTEGER NOT NULL,
                step INTEGER NOT NULL,
                template TEXT NOT NULL,
                is_default BOOLEAN NOT NULL
            );
            ",
        )?;

        // create default counter
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM counters;")?;
        if let Some(row) = stmt.iter().next() {
            println!("{:?}", row);
            if let sqlite::Value::Integer(count) = &row?[0] {
                println!("{:?}", count);
                if *count == 0 {
                    self.create_counter("tally", 0, 0, "{}", true)?;
                }
            }
        }

        Ok(())
    }

    pub fn create_counter(
        &self,
        name: &str,
        count: i64,
        step: i64,
        template: &str,
        is_default: bool,
    ) -> Result<()> {
        let mut stmt = self.conn.prepare(
            "INSERT INTO counters (name, count, step, template, is_default) 
            VALUES (?, ?, ?, ?, ?)",
        )?;

        stmt.bind((1, name))?;
        stmt.bind((2, count))?;
        stmt.bind((3, step))?;
        stmt.bind((4, template))?;
        stmt.bind((5, is_default as i64))?;

        // run the stmt
        stmt.iter().next();

        Ok(())
    }

    pub fn get_default_counter(&self) -> Result<String> {
        let mut stmt = self
            .conn
            .prepare("SELECT name FROM counters WHERE is_default = true;")?;

        // Use cursor to get the first row
        if let Some(row) = stmt.iter().next() {
            if let sqlite::Value::String(default) = &row?[0] {
                return Ok(default.clone()); // Clone the string and return it
            }
        }

        eprintln!("Error getting default counter");
        std::process::exit(1);
    }
}
