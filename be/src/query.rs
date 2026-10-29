use std::num::NonZeroU32;

use sea_query::{
    BinOper, CommonTableExpression, Condition, Expr, ExprTrait, Func, JoinType, Order,
    SelectStatement, SqliteQueryBuilder, WithClause,
};
use sea_query_sqlx::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::{AssertSqlSafe, FromRow, SqlitePool};
use vec1::Vec1;

use crate::error::AppResult;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    project_name: String,
    key: String,
    x: Field,
    select: String,
    filter: Option<Filter>,
    max_points: NonZeroU32,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum Field {
    Timestamp,
    Metric(String),
}

impl Field {
    /// Metric fields are read from the entry logged together with the selected value.
    fn expr(&self) -> Expr {
        match self {
            Self::Timestamp => Expr::col(("y", "timestamp")),
            Self::Metric(name) => SelectStatement::new()
                .column(("f", "value"))
                .from_as("metrics", "f")
                .cond_where(
                    Condition::all()
                        .add(Expr::col(("f", "run_id")).equals(("y", "run_id")))
                        .add(Expr::col(("f", "key")).equals(("y", "key")))
                        .add(Expr::col(("f", "timestamp")).equals(("y", "timestamp")))
                        .add(Expr::col(("f", "field")).eq(name.as_str())),
                )
                .to_owned()
                .into(),
        }
    }
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Filter {
    And { filters: Vec1<Filter> },
    Or { filters: Vec1<Filter> },
    Cmp { field: Field, op: CmpOp, value: f64 },
    In { field: Field, values: Vec1<f64> },
    RunIn { runs: Vec1<String> },
    RunNotIn { runs: Vec1<String> },
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CmpOp {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
}

impl CmpOp {
    fn oper(self) -> BinOper {
        match self {
            Self::Eq => BinOper::Equal,
            Self::Ne => BinOper::NotEqual,
            Self::Gt => BinOper::GreaterThan,
            Self::Gte => BinOper::GreaterThanOrEqual,
            Self::Lt => BinOper::SmallerThan,
            Self::Lte => BinOper::SmallerThanOrEqual,
        }
    }
}

#[derive(Serialize, FromRow)]
pub struct Bucket {
    x: f64,
    avg: f64,
    min: f64,
    max: f64,
    count: i64,
}

impl From<&Filter> for Condition {
    fn from(filter: &Filter) -> Self {
        let run_name = || Expr::col(("r", "run_name"));
        match filter {
            Filter::And { filters } => filters.iter().fold(Self::all(), Self::add),
            Filter::Or { filters } => filters.iter().fold(Self::any(), Self::add),
            Filter::Cmp { field, op, value } => {
                Self::all().add(field.expr().binary(op.oper(), *value))
            }
            Filter::In { field, values } => {
                Self::all().add(field.expr().is_in(values.iter().copied()))
            }
            Filter::RunIn { runs } => Self::all().add(run_name().is_in(runs.iter().cloned())),
            Filter::RunNotIn { runs } => {
                Self::all().add(run_name().is_not_in(runs.iter().cloned()))
            }
        }
    }
}

fn cte(name: &'static str, query: SelectStatement) -> CommonTableExpression {
    CommonTableExpression::new()
        .query(query)
        .table_name(name)
        .to_owned()
}

fn series(query: &Query) -> SelectStatement {
    SelectStatement::new()
        .expr_as(query.x.expr(), "x")
        .expr_as(Expr::col(("y", "value")), "value")
        .from_as("metrics", "y")
        .join_as(
            JoinType::InnerJoin,
            "runs",
            "r",
            Expr::col(("r", "id")).equals(("y", "run_id")),
        )
        .cond_where(
            Condition::all()
                .add(Expr::col(("r", "project_name")).eq(query.project_name.as_str()))
                .add(Expr::col(("y", "key")).eq(query.key.as_str()))
                .add(Expr::col(("y", "field")).eq(query.select.as_str()))
                .add(query.x.expr().is_not_null())
                .add_option(query.filter.as_ref()),
        )
        .to_owned()
}

fn bounds() -> SelectStatement {
    SelectStatement::new()
        .expr_as(Expr::col("x").min(), "lo")
        .expr_as(Expr::col("x").max().sub(Expr::col("x").min()), "span")
        .from("series")
        .to_owned()
}

pub async fn run(pool: &SqlitePool, query: &Query) -> AppResult<Vec<Bucket>> {
    let max_points = query.max_points.get();
    let bucket = Func::cust("MIN")
        .arg(Func::cast_as(
            Expr::col("x")
                .sub(Expr::col("lo"))
                .mul(max_points)
                .div(Expr::col("span")),
            "INTEGER",
        ))
        .arg(max_points - 1);

    let (sql, values) = SelectStatement::new()
        .expr_as(Expr::col("x").avg(), "x")
        .expr_as(Expr::col("value").avg(), "avg")
        .expr_as(Expr::col("value").min(), "min")
        .expr_as(Expr::col("value").max(), "max")
        .expr_as(Expr::col("value").count(), "count")
        .from("series")
        .from("bounds")
        .add_group_by([bucket.into()])
        .order_by("x", Order::Asc)
        .with_cte(
            WithClause::new()
                .cte(cte("series", series(query)))
                .cte(cte("bounds", bounds()))
                .to_owned(),
        )
        .build_sqlx(SqliteQueryBuilder);
    Ok(sqlx::query_as_with(AssertSqlSafe(sql), values)
        .fetch_all(pool)
        .await?)
}
