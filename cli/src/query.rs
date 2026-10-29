use std::num::NonZeroU32;

use anyhow::{Context, Result};
use clap::Args;
use serde_json::{Value, json};

use crate::{api::ApiClient, print};

#[derive(Args)]
pub struct QueryArgs {
    #[arg(long)]
    project: String,
    /// Metric entry key
    #[arg(long)]
    key: String,
    /// Field to aggregate
    #[arg(long)]
    select: String,
    /// X axis: `timestamp` or any logged field, e.g. `step`
    #[arg(long)]
    x: String,
    /// Only include these runs; repeat for several
    #[arg(long = "run")]
    runs: Vec<String>,
    /// Filter as JSON, in the API's format
    #[arg(long)]
    filter: Option<String>,
    /// Maximum number of buckets the x range is split into
    #[arg(long, default_value = "100")]
    max_points: NonZeroU32,
}

impl QueryArgs {
    fn filter(&self) -> Result<Option<Value>> {
        let mut filters = Vec::new();
        if let Some(filter) = &self.filter {
            filters.push(serde_json::from_str(filter).context("filter is not valid JSON")?);
        }
        if !self.runs.is_empty() {
            filters.push(json!({ "type": "run_in", "runs": self.runs }));
        }
        Ok(match filters.len() {
            0 => None,
            1 => filters.pop(),
            _ => Some(json!({ "type": "and", "filters": filters })),
        })
    }
}

fn field(name: &str) -> Value {
    match name {
        "timestamp" => json!("timestamp"),
        _ => json!({ "metric": name }),
    }
}

pub fn run(client: &ApiClient, args: QueryArgs) -> Result<()> {
    let query = json!({
        "project_name": args.project,
        "key": args.key,
        "x": field(&args.x),
        "select": args.select,
        "filter": args.filter()?,
        "max_points": args.max_points,
    });
    print(&client.post::<Value>(&["query"], &query)?)
}
