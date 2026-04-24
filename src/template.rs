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
