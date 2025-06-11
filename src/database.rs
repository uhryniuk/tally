use anyhow::Result;
use sqlite::{ConnectionThreadSafe};
use crate::models::Counter;

pub struct Connection {
    conn: ConnectionThreadSafe,
}

impl Connection {
    pub fn get(&self) -> &ConnectionThreadSafe {
        &self.conn
    }

    pub fn get_mut(&mut self) -> &mut ConnectionThreadSafe {
        &mut self.conn
    }

    pub fn new(name: &str) -> Result<Connection> {
        // Create process and thread safe connection
        let mut connection = sqlite::Connection::open_thread_safe(name).expect("ORPS");
        connection.set_busy_timeout(5_000_000)?;
        connection.execute("PRAGMA journal_mode = WAL;")?;

        let conn = Connection { conn: connection };
        conn.init_database()?;
        Ok(conn)
    }

    fn init_database(&self) -> Result<()> {
        // create the default table
        self.conn.execute(
            "
            CREATE TABLE IF NOT EXISTS counters (
                name TEXT NOT NULL,
                count INTEGER NOT NULL,
                step INTEGER NOT NULL,
                template TEXT NOT NULL
            );
            ",
        )?;

        // create default counter
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM counters;")?;
        if let Some(row) = stmt.iter().next() {
            if let sqlite::Value::Integer(count) = &row?[0] {
                if *count == 0 {
                    Counter::new("tally").insert(self.get())?;
                }
            }
        }

        Ok(())
    }

}


/// Transaction handler to ensure all transactions are either
/// completed or rolled back
pub struct Tx<'a> {
    conn: &'a ConnectionThreadSafe,
    committed: bool,
}

impl<'a> Tx<'a> {

    /// Create a new transation on the prodvided SQLite connection
    pub fn new(conn: &'a ConnectionThreadSafe) -> Result<Self> {
        conn.execute("BEGIN TRANSACTION")?;
        Ok(Self {
            conn,
            committed: false,
        })
    }

    /// Commit the transation on the prodvided SQLite connection
    pub fn commit(mut self) -> Result<()> {
        self.conn.execute("COMMIT")?;
        self.committed = true;
        Ok(())
    }
}

impl<'a> Drop for Tx<'a> {
    /// Rollback changes if the transaction is failed to be commited
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.conn.execute("ROLLBACK");
        }
    }
}
