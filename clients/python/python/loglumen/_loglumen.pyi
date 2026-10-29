from os import PathLike

def init(
    project_name: str | None = None,
    run_name: str | None = None,
    db_path: str | PathLike[str] | None = None,
) -> None: ...
def log(key: str, **fields: int | float) -> None: ...
