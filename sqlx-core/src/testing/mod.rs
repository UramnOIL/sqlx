use std::collections::HashMap;
use std::future::Future;
use std::str::FromStr;

use futures_core::future::BoxFuture;

use crate::connection::Connection;
use crate::database::Database;
use crate::error::Error;
use crate::pool::{Pool, PoolOptions};

mod fixtures;

pub use fixtures::FixtureSnapshot;

pub use sqlx_rt::test_block_on;

pub trait TestDatabase: Database {
    /// Get parameters to construct a `Pool` suitable for testing.
    ///
    /// This `Pool` instance will behave somewhat specially:
    /// * all handles share a single global semaphore to avoid exceeding the max connections
    ///   for the database flavor.
    /// * each unique invocation results in a different temporary database.
    ///
    /// The passed `ConnectOptions` will be used to manage the test databases.
    /// The user credentials it contains must have the privilege to create and drop databases.
    fn test_pool_opts<'a>(
        master_opts: <Self::Connection as Connection>::Options,
        test_path: &'a str,
    ) -> BoxFuture<'a, Result<(PoolOptions<Self>, <Self::Connection as Connection>::Options), Error>>;

    /// Cleanup any test databases that are no longer in-use.
    fn cleanup_test_dbs<'a>(
        opts: <Self::Connection as Connection>::Options,
    ) -> BoxFuture<'a, Result<(), Error>>;

    /// Take a snapshot of the current state of the database (data only).
    ///
    /// This snapshot can then be used to generate test fixtures.
    fn snapshot(conn: &mut Self::Connection)
        -> BoxFuture<'_, Result<FixtureSnapshot<Self>, Error>>;
}

pub async fn create_test_pool<DB: TestDatabase, F, Fut>(test_path: &str, build_pool: F) -> Pool<DB>
where
    F: FnOnce(PoolOptions<DB>, <DB::Connection as Connection>::Options) -> Fut,
    Fut: Future<Output = Result<Pool<DB>, Error>>,
{
    let url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set with `#[sqlx::test]`");

    let master_opts = <DB::Connection as Connection>::Options::from_str(&url)
        .expect("failed to parse DATABASE_URL");

    let (pool_opts, connect_opts) = DB::test_pool_opts(master_opts, test_path)
        .await
        .expect("failed to connect to DATABASE_URL");

    build_pool(pool_opts, connect_opts).expect("failed to build test pool")
}
