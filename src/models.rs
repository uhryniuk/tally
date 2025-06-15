use sqlite::{ConnectionThreadSafe, State, Value};

#[derive(Debug)]
pub struct Counter {
    pub name: String,
    pub count: i64,
    pub step: i64,
    pub template: String,
}

impl Counter {
    pub fn new(name: &str) -> Counter {
        Counter {
            name: name.to_string(),
            count: 0,
            step: 1,
            template: String::from("{}"),
        }
    }

    pub fn set_default(&self, conn: &ConnectionThreadSafe) -> sqlite::Result<()>{

        let mut stmt = conn.prepare("INSERT INTO default_counter (name, timestamp) VALUES (?, CURRENT_TIMESTAMP);")?;
        stmt.bind((1, self.name.as_str()))?;
        stmt.next()?;
        Ok(())
    }

    pub fn get_default(conn: &ConnectionThreadSafe) -> sqlite::Result<Option<String>> {
        let mut stmt =
            conn.prepare("SELECT name FROM default_counter ORDER BY timestamp DESC LIMIT 1;")?;
        if let State::Row = stmt.next()? {
            Ok(Some(String::from(stmt.read::<String, usize>(0)?.to_string())))
        } else {
            Ok(None)
        }
    }

    pub fn insert(&self, conn: &ConnectionThreadSafe) -> sqlite::Result<()> {
        let mut stmt =
            conn.prepare("INSERT INTO counters (name, count, step, template) VALUES (?, ?, ?, ?)")?;
        stmt.bind((1, self.name.as_str()))?;
        stmt.bind((2, self.count))?;
        stmt.bind((3, self.step))?;
        stmt.bind((4, self.template.as_str()))?;
        stmt.next()?;

        Ok(())
    }

    pub fn delete(conn: &ConnectionThreadSafe, name: &str) -> sqlite::Result<()> {
        let mut stmt = conn.prepare("DELETE FROM counters WHERE name = ?")?;
        stmt.bind((1, name))?;
        stmt.next()?;
        Ok(())
    }

    pub fn get(conn: &ConnectionThreadSafe, name: &str) -> sqlite::Result<Option<Counter>> {
        let mut stmt =
            conn.prepare("SELECT name, count, step, template FROM counters WHERE name = ?")?;
        stmt.bind((1, name))?;

        if let State::Row = stmt.next()? {
            Ok(Some(Counter {
                name: stmt.read::<String, _>(0)?.to_string(),
                count: stmt.read::<i64, _>(1)?,
                step: stmt.read::<i64, _>(2)?,
                template: stmt.read::<String, _>(3)?.to_string(),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_all(conn: &ConnectionThreadSafe) -> sqlite::Result<Vec<Counter>> {
        let mut stmt = conn.prepare("SELECT name, count, step, template FROM counters")?;
        let mut counters = Vec::new();

        while let State::Row = stmt.next()? {
            counters.push(Counter {
                name: stmt.read::<String, usize>(0)?,
                count: stmt.read::<i64, usize>(1)?,
                step: stmt.read::<i64, usize>(2)?,
                template: stmt.read::<String, usize>(3)?,
            });
        }

        Ok(counters)
    }

    pub fn update(&self, conn: &ConnectionThreadSafe) -> sqlite::Result<()> {
        let mut stmt =
            conn.prepare("UPDATE counters SET count = ?, step = ?, template = ? WHERE name = ?")?;
        stmt.bind((1, self.count))?;
        stmt.bind((2, self.step))?;
        stmt.bind((3, self.template.as_str()))?;
        stmt.bind((4, self.name.as_str()))?;
        stmt.next()?;
        Ok(())
    }
}
