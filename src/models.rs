use sqlite::{ConnectionThreadSafe, State};

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

    pub fn set_default(&self, conn: &ConnectionThreadSafe) -> sqlite::Result<()> {
        conn.execute("DELETE FROM default_counter;")?;
        let mut stmt = conn.prepare(
            "INSERT INTO default_counter (name, timestamp) VALUES (?, CURRENT_TIMESTAMP);",
        )?;
        stmt.bind((1, self.name.as_str()))?;
        stmt.next()?;
        Ok(())
    }

    pub fn get_default(conn: &ConnectionThreadSafe) -> sqlite::Result<Option<String>> {
        let mut stmt =
            conn.prepare("SELECT name FROM default_counter ORDER BY timestamp DESC LIMIT 1;")?;
        if let State::Row = stmt.next()? {
            Ok(Some(stmt.read::<String, usize>(0)?))
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Connection;
    use tempfile::TempDir;

    fn fresh_db() -> (TempDir, Connection) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let conn = Connection::new(&path.to_string_lossy()).unwrap();
        (dir, conn)
    }

    #[test]
    fn new_has_sensible_defaults() {
        let c = Counter::new("foo");
        assert_eq!(c.name, "foo");
        assert_eq!(c.count, 0);
        assert_eq!(c.step, 1);
        assert_eq!(c.template, "{}");
    }

    #[test]
    fn insert_and_get_round_trip() {
        let (_dir, conn) = fresh_db();
        let c = Counter {
            name: "foo".into(),
            count: 7,
            step: 2,
            template: "x-{}".into(),
        };
        c.insert(conn.get()).unwrap();

        let loaded = Counter::get(conn.get(), "foo").unwrap().unwrap();
        assert_eq!(loaded.name, "foo");
        assert_eq!(loaded.count, 7);
        assert_eq!(loaded.step, 2);
        assert_eq!(loaded.template, "x-{}");
    }

    #[test]
    fn get_missing_returns_none() {
        let (_dir, conn) = fresh_db();
        assert!(Counter::get(conn.get(), "nope").unwrap().is_none());
    }

    #[test]
    fn update_persists_changes() {
        let (_dir, conn) = fresh_db();
        let mut c = Counter::new("foo");
        c.insert(conn.get()).unwrap();
        c.count = 42;
        c.step = 5;
        c.update(conn.get()).unwrap();

        let loaded = Counter::get(conn.get(), "foo").unwrap().unwrap();
        assert_eq!(loaded.count, 42);
        assert_eq!(loaded.step, 5);
    }

    #[test]
    fn delete_removes_row() {
        let (_dir, conn) = fresh_db();
        Counter::new("foo").insert(conn.get()).unwrap();
        Counter::delete(conn.get(), "foo").unwrap();
        assert!(Counter::get(conn.get(), "foo").unwrap().is_none());
    }

    #[test]
    fn get_all_returns_every_counter() {
        let (_dir, conn) = fresh_db();
        Counter::new("a").insert(conn.get()).unwrap();
        Counter::new("b").insert(conn.get()).unwrap();
        let mut names: Vec<String> = Counter::get_all(conn.get())
            .unwrap()
            .into_iter()
            .map(|c| c.name)
            .collect();
        names.sort();
        assert_eq!(names, vec!["a", "b", "tally"]);
    }

    #[test]
    fn set_default_keeps_default_counter_single_row() {
        let (_dir, conn) = fresh_db();
        Counter::new("a").insert(conn.get()).unwrap();
        Counter::new("b").insert(conn.get()).unwrap();

        Counter::new("a").set_default(conn.get()).unwrap();
        Counter::new("b").set_default(conn.get()).unwrap();
        Counter::new("a").set_default(conn.get()).unwrap();

        assert_eq!(
            Counter::get_default(conn.get()).unwrap().as_deref(),
            Some("a")
        );

        let mut stmt = conn
            .get()
            .prepare("SELECT COUNT(*) FROM default_counter;")
            .unwrap();
        stmt.next().unwrap();
        let count: i64 = stmt.read(0).unwrap();
        assert_eq!(count, 1);
    }
}
