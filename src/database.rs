use crate::models::Counter;
use anyhow::Result;
use fs2::FileExt;
use sqlite::ConnectionThreadSafe;
use std::fs::File;

pub struct Connection {
    conn: ConnectionThreadSafe,
    lock_file: File,
}

impl Connection {
    pub fn get(&self) -> &ConnectionThreadSafe {
        &self.conn
    }

    pub fn new(name: &str) -> Result<Connection> {
        // Acquire file lock
        let lock_path = format!("{}.lock", name);
        let lock_file = File::create(&lock_path)?;

        if lock_file.try_lock_exclusive().is_err() {
            eprintln!("tally: waiting for another instance to release {lock_path}");
            lock_file.lock_exclusive()?;
        }

        let mut connection = sqlite::Connection::open_thread_safe(name)?;
        connection.set_busy_timeout(5_000)?;
        connection.execute("PRAGMA journal_mode = WAL;")?;

        let mut conn = Connection {
            conn: connection,
            lock_file,
        };
        conn.init_database()?;
        Ok(conn)
    }

    fn init_database(&mut self) -> Result<()> {
        // create the default table
        self.conn.execute(
            "
            CREATE TABLE IF NOT EXISTS counters (
                name TEXT PRIMARY KEY,
                count INTEGER NOT NULL,
                step INTEGER NOT NULL,
                template TEXT NOT NULL
            );
            ",
        )?;

        self.conn.execute(
            "
            CREATE TABLE IF NOT EXISTS default_counter (
                name TEXT NOT NULL,
                timestamp DATETIME NOT NULL,
                FOREIGN KEY (name) REFERENCES counters(name)
            );
            ",
        )?;

        // Setup default counter
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM counters;")?;
        if let Some(row) = stmt.iter().next() {
            if let sqlite::Value::Integer(count) = &row?[0] {
                if *count == 0 {
                    let default = Counter::new("tally");
                    default.insert(&self.conn)?;
                    default.set_default(&self.conn)?;
                }
            }
        }

        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Err(e) = fs2::FileExt::unlock(&self.lock_file) {
            eprintln!("Warning: Failed to unlock file: {}", e);
        }
    }
}
