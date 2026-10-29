use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use clap::{Args, Subcommand};
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{api::ApiClient, print};

#[derive(Args)]
pub struct ChartsArgs {
    #[command(subcommand)]
    command: ChartsCommand,
}

#[derive(Subcommand)]
enum ChartsCommand {
    /// List the charts of a dashboard
    List {
        #[command(flatten)]
        target: DashboardTarget,
    },
    /// Show a chart
    Get { id: Uuid },
    /// Add a chart to a dashboard
    Create {
        #[command(flatten)]
        target: DashboardTarget,
        #[command(flatten)]
        config: ConfigInput,
    },
    /// Replace a chart's config
    Update {
        id: Uuid,
        #[command(flatten)]
        config: ConfigInput,
    },
    /// Delete a chart
    Delete { id: Uuid },
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct DashboardTarget {
    /// Dashboard id
    #[arg(long)]
    dashboard: Option<Uuid>,
    /// Project name; uses the project's oldest dashboard
    #[arg(long)]
    project: Option<String>,
}

#[derive(Deserialize)]
struct Dashboard {
    id: Uuid,
}

impl DashboardTarget {
    fn resolve(&self, client: &ApiClient) -> Result<Option<Uuid>> {
        match (&self.dashboard, &self.project) {
            (Some(id), _) => Ok(Some(*id)),
            (None, Some(project)) => {
                let dashboards: Vec<Dashboard> =
                    client.get(&["projects", project, "dashboards"])?;
                Ok(dashboards.first().map(|dashboard| dashboard.id))
            }
            (None, None) => unreachable!("clap requires --dashboard or --project"),
        }
    }

    fn missing(&self) -> anyhow::Error {
        anyhow!(
            "project {} has no dashboard",
            self.project.as_deref().unwrap_or_default()
        )
    }
}

#[derive(Args)]
#[group(required = true, multiple = false)]
struct ConfigInput {
    /// Chart config as JSON
    #[arg(long)]
    config: Option<String>,
    /// File with the chart config as JSON
    #[arg(long)]
    config_file: Option<PathBuf>,
}

impl ConfigInput {
    fn into_value(self) -> Result<Value> {
        let text = match (self.config, self.config_file) {
            (Some(text), _) => text,
            (None, Some(path)) => std::fs::read_to_string(&path)
                .with_context(|| format!("reading {}", path.display()))?,
            (None, None) => unreachable!("clap requires --config or --config-file"),
        };
        serde_json::from_str(&text).context("config is not valid JSON")
    }
}

pub fn run(client: &ApiClient, args: ChartsArgs) -> Result<()> {
    match args.command {
        ChartsCommand::List { target } => {
            let charts = match target.resolve(client)? {
                Some(id) => client.get(&["dashboards", &id.to_string(), "charts"])?,
                None => json!([]),
            };
            print(&charts)
        }
        ChartsCommand::Get { id } => print(&client.get::<Value>(&["charts", &id.to_string()])?),
        ChartsCommand::Create { target, config } => {
            let config = config.into_value()?;
            let id = target.resolve(client)?.ok_or_else(|| target.missing())?;
            print(&client.post::<Value>(
                &["dashboards", &id.to_string(), "charts"],
                &json!({ "config": config }),
            )?)
        }
        ChartsCommand::Update { id, config } => {
            let config = config.into_value()?;
            print(&client.put::<Value>(&["charts", &id.to_string()], &json!({ "config": config }))?)
        }
        ChartsCommand::Delete { id } => client.delete(&["charts", &id.to_string()]),
    }
}
