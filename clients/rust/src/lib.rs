use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Number;
use sqlx::sqlite::{SqliteConnectOptions, SqliteConnection, SqliteJournalMode};
use sqlx::{ConnectOptions, QueryBuilder, Sqlite};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tokio::sync::mpsc;

#[doc(hidden)]
pub use serde_json;

#[derive(Debug, Error)]
pub enum LoglumenError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] sqlx::Error),
}

pub type LoglumenResult<T> = Result<T, LoglumenError>;

pub fn sqlite_db_path() -> std::path::PathBuf {
    dirs::data_dir()
        .unwrap_or_default()
        .join("loglumen")
        .join("loglumen.sqlite")
}

#[derive(Debug)]
pub struct Payload(pub Vec<(&'static str, Number)>);

// The global singleton instance
static PRODUCER: OnceLock<Producer> = OnceLock::new();

#[derive(Debug)]
struct Log {
    key: &'static str,
    timestamp: u64,
    payload: Payload,
}

struct Producer {
    tx: mpsc::Sender<Log>,
}

impl Producer {
    fn new(tx: mpsc::Sender<Log>) -> Self {
        Self { tx }
    }

    pub fn log(&self, key: &'static str, payload: Payload) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        if let Err(e) = self.tx.try_send(Log {
            key,
            timestamp,
            payload,
        }) {
            eprintln!("Log queue is full: {e}");
        }
    }
}

#[derive(Deserialize, Default)]
pub enum StorageEngine {
    #[default]
    Sqlite,
}

#[async_trait]
trait StorageEngineT {
    async fn submit(&mut self, log: Log) -> LoglumenResult<()>;
}

struct SqliteStorageEngine {
    connection: SqliteConnection,
    run_id: i64,
}

impl SqliteStorageEngine {
    async fn new(
        mut connection: SqliteConnection,
        project_name: String,
        run_name: String,
    ) -> LoglumenResult<Self> {
        // One row per payload field, so series queries are indexed scans over a REAL column.
        sqlx::raw_sql(
            r"CREATE TABLE IF NOT EXISTS runs (
              id INTEGER PRIMARY KEY,
              project_name TEXT NOT NULL,
              run_name TEXT NOT NULL,
              UNIQUE (project_name, run_name)
          );
          CREATE TABLE IF NOT EXISTS metrics (
              run_id INTEGER NOT NULL REFERENCES runs(id),
              key TEXT NOT NULL,
              field TEXT NOT NULL,
              timestamp INTEGER NOT NULL,
              value REAL NOT NULL
          );
          CREATE INDEX IF NOT EXISTS metrics_series ON metrics (run_id, key, field, timestamp);",
        )
        .execute(&mut connection)
        .await?;
        sqlx::query(
            "INSERT INTO runs (project_name, run_name) VALUES (?, ?) ON CONFLICT DO NOTHING",
        )
        .bind(&project_name)
        .bind(&run_name)
        .execute(&mut connection)
        .await?;
        let run_id =
            sqlx::query_scalar("SELECT id FROM runs WHERE project_name = ? AND run_name = ?")
                .bind(&project_name)
                .bind(&run_name)
                .fetch_one(&mut connection)
                .await?;
        Ok(Self { connection, run_id })
    }
}

#[async_trait]
impl StorageEngineT for SqliteStorageEngine {
    async fn submit(&mut self, log: Log) -> LoglumenResult<()> {
        let mut query = QueryBuilder::<Sqlite>::new(
            "INSERT INTO metrics (run_id, key, field, timestamp, value) ",
        );
        query.push_values(log.payload.0, |mut row, (field, value)| {
            row.push_bind(self.run_id)
                .push_bind(log.key)
                .push_bind(field)
                .push_bind(log.timestamp as i64)
                .push_bind(value.as_f64());
        });
        query.build().execute(&mut self.connection).await?;
        Ok(())
    }
}

async fn consumer(rx: mpsc::Receiver<Log>, mut storage_engine: impl StorageEngineT) {
    let mut rx = rx;
    loop {
        while let Some(log) = rx.recv().await {
            if let Err(e) = storage_engine.submit(log).await {
                eprintln!("{e:?}");
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Config {
    storage_engine: StorageEngine,
    project_name: String,
    run_name: String,
    channel_buffer_size: usize,
    db_path: std::path::PathBuf,
}

/// Name of the enclosing git repo's top-level directory, falling back to the cwd's name.
fn default_project_name() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| std::path::PathBuf::from(String::from_utf8_lossy(&o.stdout).trim()))
        .or_else(|| std::env::current_dir().ok())
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "unknown".to_string())
}

fn default_run_name() -> String {
    time::OffsetDateTime::now_utc()
        .format(time::macros::format_description!(
            "[year]-[month]-[day]T[hour]-[minute]-[second]"
        ))
        .expect("static format description is valid")
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage_engine: Default::default(),
            project_name: default_project_name(),
            run_name: default_run_name(),
            channel_buffer_size: 1000,
            db_path: sqlite_db_path(),
        }
    }
}

impl Config {
    pub fn storage_engine(mut self, storage_engine: StorageEngine) -> Self {
        self.storage_engine = storage_engine;
        self
    }

    pub fn project_name(mut self, project_name: String) -> Self {
        self.project_name = project_name;
        self
    }

    pub fn run_name(mut self, run_name: String) -> Self {
        self.run_name = run_name;
        self
    }

    pub fn channel_buffer_size(mut self, channel_buffer_size: usize) -> Self {
        self.channel_buffer_size = channel_buffer_size;
        self
    }

    pub fn db_path(mut self, db_path: std::path::PathBuf) -> Self {
        self.db_path = db_path;
        self
    }

    pub async fn init(self) {
        let (tx, rx) = mpsc::channel(1000);
        let client = Producer::new(tx);
        if PRODUCER.set(client).is_err() {
            eprintln!("LogLumen client is already initialized.");
        }
        if let Some(parent) = self.db_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let connection = SqliteConnectOptions::new()
            .filename(&self.db_path)
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(Duration::from_secs(5))
            .connect()
            .await
            .unwrap();
        let storage_engine = SqliteStorageEngine::new(connection, self.project_name, self.run_name)
            .await
            .unwrap();
        tokio::spawn(consumer(rx, storage_engine));
    }
}

pub fn log(key: &'static str, payload: Payload) {
    if let Some(producer) = PRODUCER.get() {
        producer.log(key, payload);
    } else {
        eprintln!("LogLumen not initialized! Dropped log: {}", key);
    }
}

#[doc(hidden)]
pub trait IntoNumber {
    fn into_number(self) -> Option<Number>;
}

macro_rules! impl_into_number {
    ($($int:ty),*; $($float:ty),*) => {
        $(impl IntoNumber for $int {
            fn into_number(self) -> Option<Number> {
                Some(Number::from(self))
            }
        })*
        $(impl IntoNumber for $float {
            fn into_number(self) -> Option<Number> {
                Number::from_f64(f64::from(self))
            }
        })*
    };
}

impl_into_number!(i8, i16, i32, i64, isize, u8, u16, u32, u64, usize; f32, f64);

#[doc(hidden)]
pub fn payload<const N: usize>(fields: [(&'static str, Option<Number>); N]) -> Payload {
    Payload(
        fields
            .into_iter()
            .filter_map(|(field, value)| {
                if value.is_none() {
                    eprintln!("Dropped non-finite value of field: {field}");
                }
                value.map(|value| (field, value))
            })
            .collect(),
    )
}

#[macro_export]
macro_rules! log {
    ($key:expr, $($field:ident = $value:expr),+ $(,)?) => {
        $crate::log(
            $key,
            $crate::payload([$((stringify!($field), $crate::IntoNumber::into_number($value))),+]),
        )
    };
}
