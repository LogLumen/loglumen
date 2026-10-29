use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use clap::{Args, Subcommand};
use flate2::read::GzDecoder;
use reqwest::{StatusCode, blocking::Client};

use crate::api::ApiArgs;

const BIN: &str = "loglumen-server";
const REPO: &str = "LogLumen/loglumen";
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Args)]
pub struct ServerArgs {
    #[command(subcommand)]
    command: ServerCommand,
}

#[derive(Subcommand)]
enum ServerCommand {
    /// Start the server, downloading it first if it isn't installed
    Start(StartArgs),
    /// Stop the detached server
    Stop,
    /// Show whether the detached server is running
    Status,
    /// Print the detached server's log
    Logs,
}

#[derive(Args)]
struct StartArgs {
    /// Run in the background; manage it with `server stop`, `status` and `logs`
    #[arg(long, short)]
    detached: bool,
    /// Port to listen on; the server defaults to 3000
    #[arg(long)]
    port: Option<u16>,
    /// SQLite URL of the server's own database
    #[arg(long)]
    database_url: Option<String>,
    /// Leave the UI and its endpoints open without a password
    #[arg(long)]
    disable_basic_auth: bool,
    /// Accept logs without an API key
    #[arg(long)]
    disable_api_key_check: bool,
    /// Download the server even if one is already installed
    #[arg(long)]
    update: bool,
}

struct Paths {
    dir: PathBuf,
    bin: PathBuf,
    pid: PathBuf,
    log: PathBuf,
}

impl Paths {
    fn new() -> Result<Self> {
        let dir = dirs::data_dir()
            .context("cannot find the data directory")?
            .join("loglumen");
        Ok(Self {
            bin: dir.join("bin").join(BIN),
            pid: dir.join("server.pid"),
            log: dir.join("server.log"),
            dir,
        })
    }
}

pub fn run(api: &ApiArgs, args: ServerArgs) -> Result<()> {
    let paths = Paths::new()?;
    match args.command {
        ServerCommand::Start(args) => start(api, args, &paths),
        ServerCommand::Stop => stop(&paths),
        ServerCommand::Status => {
            match running_pid(&paths.pid) {
                Some(pid) => println!("running (pid {pid}), logs: {}", paths.log.display()),
                None => println!("not running"),
            }
            Ok(())
        }
        ServerCommand::Logs => {
            let mut log = File::open(&paths.log)
                .with_context(|| format!("no log at {}", paths.log.display()))?;
            io::copy(&mut log, &mut io::stdout())?;
            Ok(())
        }
    }
}

fn start(api: &ApiArgs, args: StartArgs, paths: &Paths) -> Result<()> {
    if let Some(pid) = running_pid(&paths.pid) {
        bail!("already running (pid {pid})");
    }
    let bin = server_binary(&paths.bin, args.update)?;
    let mut command = Command::new(&bin);
    if let Some(password) = api.admin_password() {
        command.env("ADMIN_PASSWORD", password);
    }
    if let Some(port) = args.port {
        command.env("PORT", port.to_string());
    }
    if let Some(url) = &args.database_url {
        command.env("DATABASE_URL", url);
    }
    if args.disable_basic_auth {
        command.env("DISABLE_BASIC_AUTH", "true");
    }
    if args.disable_api_key_check {
        command.env("DISABLE_API_KEY_CHECK", "true");
    }

    if !args.detached {
        return Err(command.exec()).with_context(|| format!("running {}", bin.display()));
    }

    fs::create_dir_all(&paths.dir).with_context(|| format!("creating {}", paths.dir.display()))?;
    let log = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.log)
        .with_context(|| format!("opening {}", paths.log.display()))?;
    let offset = log.metadata()?.len();
    let mut child = command
        .stdin(Stdio::null())
        .stdout(log.try_clone()?)
        .stderr(log)
        .process_group(0)
        .spawn()
        .with_context(|| format!("running {}", bin.display()))?;
    fs::write(&paths.pid, child.id().to_string())?;

    sleep(Duration::from_millis(500));
    if let Some(status) = child.try_wait()? {
        fs::remove_file(&paths.pid)?;
        let mut output = String::new();
        let mut log = File::open(&paths.log)?;
        log.seek(SeekFrom::Start(offset))?;
        log.read_to_string(&mut output)?;
        bail!("{BIN} failed to start ({status}):\n{}", output.trim_end());
    }

    let port = args
        .port
        .or_else(|| std::env::var("PORT").ok()?.parse().ok())
        .unwrap_or(3000);
    println!(
        "started {BIN} (pid {}) at http://localhost:{port}",
        child.id()
    );
    println!("logs: {}", paths.log.display());
    Ok(())
}

fn stop(paths: &Paths) -> Result<()> {
    let Some(pid) = running_pid(&paths.pid) else {
        println!("not running");
        return Ok(());
    };
    if !kill(&[], pid) {
        bail!("failed to stop {BIN} (pid {pid})");
    }
    for _ in 0..50 {
        if !kill(&["-0"], pid) {
            fs::remove_file(&paths.pid)?;
            println!("stopped (pid {pid})");
            return Ok(());
        }
        sleep(Duration::from_millis(100));
    }
    bail!("{BIN} (pid {pid}) did not stop within 5s")
}

fn running_pid(pid_file: &Path) -> Option<u32> {
    let pid = fs::read_to_string(pid_file).ok()?.trim().parse().ok();
    match pid {
        Some(pid) if kill(&["-0"], pid) => Some(pid),
        _ => {
            let _ = fs::remove_file(pid_file);
            None
        }
    }
}

fn kill(args: &[&str], pid: u32) -> bool {
    Command::new("kill")
        .args(args)
        .arg(pid.to_string())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn server_binary(managed: &Path, update: bool) -> Result<PathBuf> {
    let path = std::env::var_os("PATH").unwrap_or_default();
    let candidates = std::iter::once(std::env::current_exe()?.with_file_name(BIN))
        .chain(std::env::split_paths(&path).map(|dir| dir.join(BIN)))
        .chain(std::iter::once(managed.to_path_buf()));
    if let Some(found) = candidates.filter(|_| !update).find(|path| path.is_file()) {
        return Ok(found);
    }
    download(managed)?;
    Ok(managed.to_path_buf())
}

fn target() -> Result<&'static str> {
    use std::env::consts::{ARCH, OS};
    Ok(match (OS, ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-musl",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        _ => bail!("no prebuilt {BIN} for {OS}/{ARCH}"),
    })
}

fn download(dest: &Path) -> Result<()> {
    let target = target()?;
    let url =
        format!("https://github.com/{REPO}/releases/download/v{VERSION}/loglumen-{target}.tar.gz");
    println!("downloading {url}");
    let response = Client::builder().timeout(None).build()?.get(&url).send()?;
    if response.status() == StatusCode::NOT_FOUND {
        bail!("no {BIN} v{VERSION} release for {target}");
    }
    let response = response.error_for_status()?;

    let dir = dest.parent().context("invalid install path")?;
    fs::create_dir_all(dir).with_context(|| format!("creating {}", dir.display()))?;
    let tmp = dest.with_extension("tmp");
    let mut archive = tar::Archive::new(GzDecoder::new(response));
    for entry in archive.entries()? {
        let mut entry = entry?;
        if *entry.path()? == *Path::new(BIN) {
            entry.unpack(&tmp)?;
            fs::set_permissions(&tmp, fs::Permissions::from_mode(0o755))?;
            fs::rename(&tmp, dest)?;
            println!("downloaded {BIN} v{VERSION} to {}", dest.display());
            return Ok(());
        }
    }
    bail!("{url} does not contain {BIN}")
}
