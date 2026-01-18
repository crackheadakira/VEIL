use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{
    CachedStatement, MappedRows, Params, Result as SqlResult, Row, Statement, Transaction,
};
use std::time::Instant;

#[cfg(debug_assertions)]
macro_rules! bench {
    ($sql:expr, $block:expr) => {{
        let start = Instant::now();
        let res = $block;
        let elapsed = start.elapsed();
        logging::debug!("{} took {:?}", $sql, elapsed);
        res
    }};
}

#[cfg(not(debug_assertions))]
macro_rules! bench {
    ($sql:expr, $block:expr) => {
        $block
    };
}
pub struct TimedConnection(PooledConnection<SqliteConnectionManager>);

impl TimedConnection {
    /// prepare a statement
    pub fn prepare(&self, sql: &str) -> SqlResult<TimedStatement<'_>> {
        let stmt = bench!(sql, self.0.prepare(sql))?;
        Ok(TimedStatement(stmt, sql.to_owned()))
    }

    /// prepare a cached statement
    pub fn prepare_cached(&self, sql: &str) -> SqlResult<TimedCachedStatement<'_>> {
        let stmt = bench!(sql, self.0.prepare_cached(sql))?;
        Ok(TimedCachedStatement(stmt, sql.to_owned()))
    }

    /// execute a statement
    pub fn execute<P: Params>(&self, sql: &str, params: P) -> SqlResult<usize> {
        bench!(sql, self.0.execute(sql, params))
    }

    /// execute a batch of statements
    pub fn execute_batch(&self, sql: &str) -> SqlResult<()> {
        bench!(sql, self.0.execute_batch(sql))
    }

    pub fn transaction(&mut self) -> rusqlite::Result<Transaction<'_>> {
        self.0.transaction()
    }
}

pub struct TimedCachedStatement<'conn>(CachedStatement<'conn>, String);

impl<'conn> TimedCachedStatement<'conn> {
    pub fn execute<P: Params>(&mut self, params: P) -> SqlResult<usize> {
        bench!(self.1, self.0.execute(params))
    }

    pub fn query_row<P, F, T>(&mut self, params: P, f: F) -> SqlResult<T>
    where
        P: Params,
        F: FnOnce(&Row<'_>) -> SqlResult<T>,
    {
        bench!(self.1, self.0.query_row(params, f))
    }

    pub fn query_map<P, F, T>(
        &mut self,
        params: P,
        f: F,
    ) -> Result<MappedRows<'_, F>, rusqlite::Error>
    where
        P: Params,
        F: FnMut(&Row<'_>) -> SqlResult<T>,
    {
        bench!(self.1, self.0.query_map(params, f))
    }
}

pub struct TimedStatement<'conn>(Statement<'conn>, String);

impl<'conn> TimedStatement<'conn> {
    pub fn execute<P: Params>(&mut self, params: P) -> SqlResult<usize> {
        bench!(self.1, self.0.execute(params))
    }

    pub fn query_row<P, F, T>(&mut self, params: P, f: F) -> SqlResult<T>
    where
        P: Params,
        F: FnOnce(&Row<'_>) -> SqlResult<T>,
    {
        bench!(self.1, self.0.query_row(params, f))
    }

    pub fn exists<P: Params>(&mut self, params: P) -> SqlResult<bool> {
        bench!(self.1, self.0.exists(params))
    }

    pub fn query_map<P, F, T>(
        &mut self,
        params: P,
        f: F,
    ) -> Result<MappedRows<'_, F>, rusqlite::Error>
    where
        P: Params,
        F: FnMut(&Row<'_>) -> SqlResult<T>,
    {
        bench!(self.1, self.0.query_map(params, f))
    }
}

pub struct TimedPool(pub Pool<SqliteConnectionManager>);

impl TimedPool {
    pub fn get(&self) -> Result<TimedConnection, r2d2::Error> {
        let conn = self.0.get()?;
        Ok(TimedConnection(conn))
    }
}
