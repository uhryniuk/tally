use crate::database::Connection;
use crate::models::Counter;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::sync::LazyLock;

static TEMPLATE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\{(.*?)\}").unwrap());

pub fn render(conn: &Connection, name: &str) -> Result<String> {
    // parse template, finding replace
    let counter = match Counter::get(conn.get(), name)? {
        Some(c) => c,
        None => return Err(anyhow!("Unable to find counter for templating")),
    };

    let mut rendered = counter.template.replace("{}", &counter.count.to_string());

    // Find all matches
    for cap in TEMPLATE_RE.captures_iter(&rendered.clone()) {
        // replace {value} with {}
        rendered = rendered.replace(&cap[0], "{}");
        // get recursive template
        let sub_template = render(conn, &cap[1])?;
        // render all recursed template
        rendered = rendered.replace("{}", &sub_template);
    }

    Ok(rendered)
}
