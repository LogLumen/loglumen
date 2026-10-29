use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::Value;

use crate::{api::ApiClient, print};

#[derive(Args)]
pub struct ProjectsArgs {
    #[command(subcommand)]
    command: ProjectsCommand,
}

#[derive(Subcommand)]
enum ProjectsCommand {
    /// List the projects that have logged runs
    List,
}

#[derive(Args)]
pub struct RunsArgs {
    #[command(subcommand)]
    command: RunsCommand,
}

#[derive(Subcommand)]
enum RunsCommand {
    /// List the runs of a project
    List {
        #[arg(long)]
        project: String,
    },
}

#[derive(Args)]
pub struct MetricsArgs {
    #[command(subcommand)]
    command: MetricsCommand,
}

#[derive(Subcommand)]
enum MetricsCommand {
    /// List the metric entry keys logged in a run
    List {
        #[command(flatten)]
        run: RunTarget,
    },
}

#[derive(Args)]
pub struct FieldsArgs {
    #[command(subcommand)]
    command: FieldsCommand,
}

#[derive(Subcommand)]
enum FieldsCommand {
    /// List the fields logged under a metric key in a run
    List {
        #[command(flatten)]
        run: RunTarget,
        /// Metric entry key
        #[arg(long)]
        key: String,
    },
}

#[derive(Args)]
struct RunTarget {
    #[arg(long)]
    project: String,
    #[arg(long)]
    run: String,
}

pub fn projects(client: &ApiClient, args: ProjectsArgs) -> Result<()> {
    match args.command {
        ProjectsCommand::List => print(&client.get::<Value>(&["projects"])?),
    }
}

pub fn runs(client: &ApiClient, args: RunsArgs) -> Result<()> {
    match args.command {
        RunsCommand::List { project } => {
            print(&client.get::<Value>(&["projects", &project, "runs"])?)
        }
    }
}

pub fn metrics(client: &ApiClient, args: MetricsArgs) -> Result<()> {
    match args.command {
        MetricsCommand::List { run } => {
            print(&client.get::<Value>(&["projects", &run.project, "runs", &run.run, "metrics"])?)
        }
    }
}

pub fn fields(client: &ApiClient, args: FieldsArgs) -> Result<()> {
    match args.command {
        FieldsCommand::List { run, key } => print(&client.get::<Value>(&[
            "projects",
            &run.project,
            "runs",
            &run.run,
            "metrics",
            &key,
            "fields",
        ])?),
    }
}
