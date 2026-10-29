mod api;
mod catalog;
mod charts;
mod query;
mod skill;

use anyhow::Result;
use api::ApiArgs;
use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Parser)]
#[command(about = "Configure LogLumen through its API")]
struct Cli {
    #[command(flatten)]
    api: ApiArgs,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List projects
    Projects(catalog::ProjectsArgs),
    /// List a project's runs
    Runs(catalog::RunsArgs),
    /// List a run's metric entry keys
    Metrics(catalog::MetricsArgs),
    /// List the fields logged under a metric key
    Fields(catalog::FieldsArgs),
    /// Aggregate logged values into buckets along an x axis
    Query(query::QueryArgs),
    /// Manage dashboard charts
    Charts(charts::ChartsArgs),
    /// Install the agent skill for coding agents
    Skill(skill::SkillArgs),
}

fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let cli = Cli::parse();
    match cli.command {
        Command::Projects(args) => catalog::projects(&cli.api.client()?, args),
        Command::Runs(args) => catalog::runs(&cli.api.client()?, args),
        Command::Metrics(args) => catalog::metrics(&cli.api.client()?, args),
        Command::Fields(args) => catalog::fields(&cli.api.client()?, args),
        Command::Query(args) => query::run(&cli.api.client()?, args),
        Command::Charts(args) => charts::run(&cli.api.client()?, args),
        Command::Skill(args) => skill::run(args),
    }
}

fn print(value: &impl Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
