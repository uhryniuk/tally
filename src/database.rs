use anyhow::Result;
use sqlite::{ConnectionThreadSafe, Value};

#[derive(Debug)]
pub struct Counter {
    pub name: String,
    pub count: i64,
    pub step: i64,
    pub template: String,
    pub is_default: bool,
}

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
            if let sqlite::Value::Integer(count) = &row?[0] {
                if *count == 0 {
                    self.create_counter("tally", 0, 1, "{}", true)?;
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

    pub fn get_count(&self, name: &str) -> Result<i64> {
        // get write lock
        self.conn.execute("BEGIN TRANSACTION;")?;

        let mut stmt = self
            .conn
            .prepare("SELECT count FROM counters WHERE name = ?")?;

        stmt.bind((1, name))?;

        // return count if exists
        if let Some(row) = stmt.iter().next() {
            if let sqlite::Value::Integer(count) = &row?[0] {
                self.conn.execute("COMMIT;")?;
                return Ok(*count);
            }
        }

        // release write lock
        self.conn.execute("COMMIT;")?;

        Ok(0)
    }

    pub fn init_counter(&self, name: &str) -> Result<()> {
        self.conn.execute("BEGIN TRANSACTION;")?;
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM counters WHERE name = ?;")?;
        stmt.bind((1, name))?;

        if let Ok(sqlite::State::Row) = stmt.next() {
            self.conn.execute("COMMIT;")?;
            return Ok(());
        }

        let mut stmt = self.conn.prepare(
            "INSERT INTO counters (name, count, step, template, is_default) 
             VALUES (?, ?, ?, ?, ?)",
        )?;
        stmt.bind((1, name))?;
        stmt.bind((2, 0))?;
        stmt.bind((3, 1))?;
        stmt.bind((4, "{}"))?;
        stmt.bind((5, false as i64))?;
        stmt.next()?;
        self.conn.execute("COMMIT;")?;

        Ok(())
    }

    fn update_and_get_count(&self, name: &str, amount: i64, op: char) -> Result<i64> {
        // Start a transaction to lock the table
        self.conn.execute("BEGIN TRANSACTION;")?;

        let mut update_stmt = match op {
            '+' => self
                .conn
                .prepare("UPDATE counters SET count = count + ? WHERE name = ?;")?,
            '-' => self
                .conn
                .prepare("UPDATE counters SET count = count - ? WHERE name = ?;")?,
            _ => {
                eprintln!("Couldn't update count");
                std::process::exit(1);
            }
        };

        // Bind and execute the update
        update_stmt.bind((1, amount))?;
        update_stmt.bind((2, name))?;
        update_stmt.next()?;

        // get the updates row
        let mut query_stmt = self
            .conn
            .prepare("SELECT count FROM counters WHERE name = ?;")?;
        query_stmt.bind((1, name))?;

        let row = query_stmt.iter().next().unwrap();
        let count: i64 = match &row?[0] {
            Value::Integer(count) => *count,
            _ => {
                eprintln!("couldn't get updated count");
                std::process::exit(1);
            }
        };

        // Commit the transaction
        self.conn.execute("COMMIT;")?;

        Ok(count)
    }

    pub fn increment_and_get_count(&self, name: &str, amount: i64) -> Result<i64> {
        self.update_and_get_count(name, amount, '+')
    }

    pub fn decrement_and_get_count(&self, name: &str, amount: i64) -> Result<i64> {
        self.update_and_get_count(name, amount, '-')
    }

    /// Delete a counter using the provided name
    pub fn delete_counter(&self, name: &str) -> Result<()> {
        self.conn.execute("BEGIN TRANSACTION;")?;

        let prior = self.get_all_counters()?.len();

        // delete the counter
        let mut stmt = self.conn.prepare("DELETE FROM counters WHERE name = ?;")?;
        stmt.bind((1, name))?;
        stmt.next()?;

        let post = self.get_all_counters()?.len();

        self.conn.execute("COMMIT;")?;

        // TODO shouldn't be able to delete default counter
        if post < prior {
            eprintln!("Counter '{}' has been deleted.", name);
        } else {
            eprintln!("Counter '{}' does not exist.", name);
        }

        Ok(())
    }

    /// Get all the counters that exist
    pub fn get_all_counters(&self) -> Result<Vec<Counter>> {
        let mut stmt = self.conn.prepare("SELECT * FROM counters")?;
        let mut rows: Vec<Counter> = Vec::new();

        // Iterate over the rows in the query result
        while let Ok(sqlite::State::Row) = stmt.next() {
            // extract values from row in stmt
            let name = stmt.read::<String, usize>(0)?;
            let count = stmt.read::<i64, usize>(1)?;
            let step = stmt.read::<i64, usize>(2)?;
            let template = stmt.read::<String, usize>(3)?;
            let is_default = stmt.read::<i64, usize>(4)?;

            rows.push(Counter {
                name,
                count,
                step,
                template,
                is_default: is_default != 0,
            })
        }

        Ok(rows)
    }

    pub fn get_step(&self, name: &str) -> Result<i64> {
        let mut stmt = self
            .conn
            .prepare("SELECT step FROM counters WHERE name = ?;")?;
        stmt.bind((1, name))?;

        if let Ok(sqlite::State::Row) = stmt.next() {
            let step = stmt.read::<i64, usize>(0)?;
            return Ok(step);
        }

        Err(anyhow::anyhow!("Unable to get 'step' value for '{}'", name))
    }

    pub fn set_count(&self, name: &str, count: i64) -> Result<()> {
        // Start a transaction to lock the table
        self.conn.execute("BEGIN TRANSACTION;")?;
        let mut stmt = self
            .conn
            .prepare("UPDATE counters SET count = ? WHERE name = ?;")?;

        // Bind and execute the update
        stmt.bind((1, count))?;
        stmt.bind((2, name))?;
        stmt.next()?;

        // Commit the transaction
        self.conn.execute("COMMIT;")?;

        Ok(())
    }

    pub fn set_step(&self, name: &str, step: i64) -> Result<()> {
        // Start a transaction to lock the table
        self.conn.execute("BEGIN TRANSACTION;")?;
        let mut stmt = self
            .conn
            .prepare("UPDATE counters SET step = ? WHERE name = ?;")?;

        // Bind and execute the update
        stmt.bind((1, step))?;
        stmt.bind((2, name))?;
        stmt.next()?;

        // Commit the transaction
        self.conn.execute("COMMIT;")?;

        Ok(())
    }

    pub fn set_template(&self, name: &str, template: &str) -> Result<()> {
        // Start a transaction to lock the table
        self.conn.execute("BEGIN TRANSACTION;")?;
        let mut stmt = self
            .conn
            .prepare("UPDATE counters SET template = ? WHERE name = ?;")?;

        // Bind and execute the update
        stmt.bind((1, template))?;
        stmt.bind((2, name))?;
        stmt.next()?;

        // Commit the transaction
        self.conn.execute("COMMIT;")?;

        Ok(())
    }

    pub fn set_default(&self, name: &str, default: bool) -> Result<()> {
        self.conn.execute("BEGIN TRANSACTION;")?;

        // set all is_default to false
        let mut stmt = self.conn.prepare("UPDATE counters SET is_default = ?;")?;
        stmt.bind((1, false as i64))?;
        stmt.next()?;

        // set given to true
        let mut stmt = self
            .conn
            .prepare("UPDATE counters SET is_default = ? WHERE name = ?;")?;

        stmt.bind((1, default as i64))?;
        stmt.bind((2, name))?;
        stmt.next()?;

        // Commit the transaction
        self.conn.execute("COMMIT;")?;

        Ok(())
    }

    pub fn get_template(&self, name: &str) -> Result<String> {
        // get write lock
        self.conn.execute("BEGIN TRANSACTION;")?;

        let mut stmt = self
            .conn
            .prepare("SELECT template FROM counters WHERE name = ?")?;

        stmt.bind((1, name))?;

        // return count if exists
        if let Some(row) = stmt.iter().next() {
            if let sqlite::Value::String(template) = &row?[0] {
                self.conn.execute("COMMIT;")?;
                return Ok(template.clone());
            }
        }

        self.conn.execute("COMMIT;")?;
        Err(anyhow::anyhow!("Couldn't get template for '{}'", name))
    }
}
