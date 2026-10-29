use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Args, Subcommand, ValueEnum};

const SKILL: &str = include_str!("../skill/SKILL.md");
const NAME: &str = "loglumen-cli";

#[derive(Args)]
pub struct SkillArgs {
    #[command(subcommand)]
    command: SkillCommand,
}

#[derive(Subcommand)]
enum SkillCommand {
    /// Write the skill into coding agents' skills directories
    Install {
        /// Agent to install for; repeat for several
        #[arg(long = "agent", value_enum, default_value = "claude")]
        agents: Vec<Agent>,
        /// Install into the current directory instead of the home directory
        #[arg(long)]
        project: bool,
    },
    /// Print the skill
    Print,
}

#[derive(ValueEnum, Clone)]
enum Agent {
    /// ~/.claude/skills or .claude/skills
    Claude,
    /// Same as agents
    Codex,
    /// ~/.cursor/skills or .cursor/skills
    Cursor,
    /// ~/.agents/skills or .agents/skills
    Agents,
}

impl Agent {
    fn dir(&self) -> &'static str {
        match self {
            Agent::Claude => ".claude",
            Agent::Cursor => ".cursor",
            Agent::Codex | Agent::Agents => ".agents",
        }
    }
}

pub fn run(args: SkillArgs) -> Result<()> {
    match args.command {
        SkillCommand::Install { agents, project } => install(&agents, project),
        SkillCommand::Print => {
            print!("{SKILL}");
            Ok(())
        }
    }
}

fn install(agents: &[Agent], project: bool) -> Result<()> {
    let root = if project {
        std::env::current_dir()?
    } else {
        std::env::home_dir().context("cannot find the home directory")?
    };
    let mut dirs: Vec<PathBuf> = agents
        .iter()
        .map(|agent| root.join(agent.dir()).join("skills").join(NAME))
        .collect();
    dirs.sort();
    dirs.dedup();
    for dir in dirs {
        std::fs::create_dir_all(&dir).with_context(|| format!("creating {}", dir.display()))?;
        let path = dir.join("SKILL.md");
        std::fs::write(&path, SKILL).with_context(|| format!("writing {}", path.display()))?;
        println!("installed {}", path.display());
    }
    Ok(())
}
