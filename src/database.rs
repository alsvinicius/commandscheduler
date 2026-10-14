//! SQLx's SQLite API without optional server drivers or their dependencies.
pub use sqlx_core::{
    error::Error, from_row::FromRow, query::query, query_as::query_as, query_builder::QueryBuilder,
    query_scalar::query_scalar, row::Row, transaction::Transaction,
};
pub use sqlx_sqlite::{self as sqlite, Sqlite, SqlitePool};
