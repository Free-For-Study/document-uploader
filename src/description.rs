use std::str::FromStr;

use anyhow::Context;

#[derive(Debug)]
pub struct Description {
    pub name: String,
    pub category: String,
}
impl FromStr for Description {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let name = lines.next().context("Failed to get first line")?;
        let (_, name) = name.split_once(':').context("Invalid name format")?;

        let category = lines.next().context("Failed to get first line")?;
        let (_, category) = category.split_once(':').context("Invalid name format")?;

        Ok(Description {
            name: name.trim().to_string(),
            category: category.trim().to_string(),
        })
    }
}
