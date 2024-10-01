use crate::database::Database;
use anyhow::Result;
use regex::Regex;

pub fn render(db: &Database, name: &str) -> Result<String> {
    // parse template, finding replace
    let template = db.get_template(&name)?;
    let count = db.get_count(&name)?;
    let mut rendered = template.replace("{}", &count.to_string());

    // Create the regex pattern to match anything inside {}
    let re = Regex::new(r"\{(.*?)\}").unwrap();

    // Find all matches
    for cap in re.captures_iter(&rendered.clone()) {
        // replace {value} with {}
        rendered = rendered.replace(&cap[0], "{}");
        // get recursive template
        let sub_template = render(db, &cap[1])?;
        // render all recursed template
        rendered = rendered.replace("{}", &sub_template);
    }
    Ok(rendered)
}
