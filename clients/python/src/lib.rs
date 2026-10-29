use pyo3::prelude::*;

#[pymodule]
mod _loglumen {
    use pyo3::exceptions::PyTypeError;
    use pyo3::prelude::*;
    use pyo3::types::PyDict;
    use serde_json::Number;
    use std::path::PathBuf;
    use std::sync::OnceLock;
    use tokio::runtime::Runtime;

    static RUNTIME: OnceLock<Runtime> = OnceLock::new();

    #[pyfunction]
    #[pyo3(signature = (project_name=None, run_name=None, db_path=None))]
    fn init(
        py: Python<'_>,
        project_name: Option<String>,
        run_name: Option<String>,
        db_path: Option<PathBuf>,
    ) {
        let mut config = loglumen::Config::default();
        if let Some(project_name) = project_name {
            config = config.project_name(project_name);
        }
        if let Some(run_name) = run_name {
            config = config.run_name(run_name);
        }
        if let Some(db_path) = db_path {
            config = config.db_path(db_path);
        }
        let runtime = RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .build()
                .expect("failed to build tokio runtime")
        });
        py.detach(|| runtime.block_on(config.init()));
    }

    #[pyfunction]
    #[pyo3(signature = (key, **fields))]
    fn log(key: String, fields: Option<&Bound<'_, PyDict>>) -> PyResult<()> {
        let mut payload = Vec::new();
        for (field, value) in fields.into_iter().flatten() {
            let field: String = field.extract()?;
            let number = if let Ok(value) = value.extract::<i64>() {
                Number::from(value)
            } else if let Ok(value) = value.extract::<u64>() {
                Number::from(value)
            } else if let Ok(value) = value.extract::<f64>() {
                match Number::from_f64(value) {
                    Some(number) => number,
                    None => {
                        eprintln!("Dropped non-finite value of field: {field}");
                        continue;
                    }
                }
            } else {
                return Err(PyTypeError::new_err(format!(
                    "field `{field}` must be int or float, got {}",
                    value.get_type().name()?
                )));
            };
            payload.push((field.into(), number));
        }
        loglumen::log(key, loglumen::Payload(payload));
        Ok(())
    }
}
