use std::path::PathBuf;

pub fn sqlite_db_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_default()
        .join("loglumen")
        .join("loglumen.sqlite")
}
