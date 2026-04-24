use crate::database::Connection;
use crate::models::Counter;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static TEMPLATE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{(.*?)\}").unwrap());

pub fn render(conn: &Connection, name: &str) -> Result<String> {
    let mut visited = HashSet::new();
    render_inner(conn, name, &mut visited)
}

fn render_inner(
    conn: &Connection,
    name: &str,
    visited: &mut HashSet<String>,
) -> Result<String> {
    if !visited.insert(name.to_string()) {
        return Err(anyhow!(
            "template cycle detected involving counter '{}'",
            name
        ));
    }

    let counter = match Counter::get(conn.get(), name)? {
        Some(c) => c,
        None => return Err(anyhow!("Unable to find counter for templating")),
    };

    let mut rendered = counter.template.replace("{}", &counter.count.to_string());

    for cap in TEMPLATE_RE.captures_iter(&rendered.clone()) {
        rendered = rendered.replace(&cap[0], "{}");
        let sub_template = render_inner(conn, &cap[1], visited)?;
        rendered = rendered.replace("{}", &sub_template);
    }

    visited.remove(name);
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Connection;
    use crate::models::Counter;
    use tempfile::TempDir;

    fn fresh_db() -> (TempDir, Connection) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.db");
        let conn = Connection::new(&path.to_string_lossy()).unwrap();
        (dir, conn)
    }

    fn put(conn: &Connection, name: &str, count: i64, template: &str) {
        let c = Counter {
            name: name.into(),
            count,
            step: 1,
            template: template.into(),
        };
        c.insert(conn.get()).unwrap();
    }

    #[test]
    fn renders_plain_counter() {
        let (_dir, conn) = fresh_db();
        put(&conn, "a", 5, "{}");
        assert_eq!(render(&conn, "a").unwrap(), "5");
    }

    #[test]
    fn renders_template_with_literal_text() {
        let (_dir, conn) = fresh_db();
        put(&conn, "a", 3, "v{}");
        assert_eq!(render(&conn, "a").unwrap(), "v3");
    }

    #[test]
    fn renders_nested_reference() {
        let (_dir, conn) = fresh_db();
        put(&conn, "inner", 7, "{}");
        put(&conn, "outer", 0, "[{inner}]");
        assert_eq!(render(&conn, "outer").unwrap(), "[7]");
    }

    #[test]
    fn missing_counter_errors() {
        let (_dir, conn) = fresh_db();
        assert!(render(&conn, "ghost").is_err());
    }

    #[test]
    fn direct_self_cycle_errors() {
        let (_dir, conn) = fresh_db();
        put(&conn, "a", 0, "{a}");
        let err = render(&conn, "a").unwrap_err().to_string();
        assert!(err.contains("cycle"), "got: {err}");
    }

    #[test]
    fn indirect_cycle_errors() {
        let (_dir, conn) = fresh_db();
        put(&conn, "a", 0, "{b}");
        put(&conn, "b", 0, "{a}");
        let err = render(&conn, "a").unwrap_err().to_string();
        assert!(err.contains("cycle"), "got: {err}");
    }

    #[test]
    fn sibling_references_are_not_cycles() {
        let (_dir, conn) = fresh_db();
        put(&conn, "leaf", 9, "{}");
        put(&conn, "root", 0, "{leaf}-{leaf}");
        assert_eq!(render(&conn, "root").unwrap(), "9-9");
    }
}
