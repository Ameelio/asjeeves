use std::fmt::Display;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

#[cfg(feature = "bb8")]
use metrics::gauge;
use metrics::histogram;

static DB_CONFIG: OnceLock<DbConfig> = OnceLock::new();

pub fn db_metrics_setup(name: impl Into<Arc<str>>, system: impl Into<Arc<str>>) {
    let name: Arc<str> = name.into();
    let system: Arc<str> = system.into();

    let config = DbConfig { name, system };

    let _ = DB_CONFIG.set(config);
}

pub async fn time_async_query<Closure, Fut, RetVal>(
    query_name: impl Display,
    query: Closure,
) -> RetVal
where
    Closure: FnOnce() -> Fut,
    Fut: Future<Output = RetVal>,
{
    let timer = Instant::now();

    let result = query().await;

    let system_name: &str = match DB_CONFIG.get() {
        Some(config) => &config.system,
        None => "",
    };

    let histogram = histogram!(
        description: "Query execution time",
        unit: metrics::Unit::Seconds,
        "db.client.operation.duration",
        "db.query.summary" => query_name.to_string(),
        "db.system.name" => system_name
    );

    histogram.record(timer.elapsed().as_secs_f64());

    result
}

#[cfg(feature = "bb8")]
pub fn track_database_metrics(dbstate: bb8::State, pool_size: u32) {
    let Some(config) = DB_CONFIG.get() else {
        return;
    };

    let stats: bb8::Statistics = dbstate.statistics;

    let active_connections: u32 = dbstate.connections - dbstate.idle_connections;

    let pool_name: &str = &config.name;

    let conn_count = gauge!(
        description: "Active Connections",
        unit: metrics::Unit::Count,
        "db.client.connection.count",
        "db.client.connetion.pool.name" => pool_name,
        "db.client.connection.state" => "used",
    );

    let idle_count = gauge!(
        description: "Idle Connections",
        unit: metrics::Unit::Count,
        "db.client.connection.count",
        "db.client.connetion.pool.name" => pool_name,
        "db.client.connection.state" => "idle",
    );

    let max_conn = gauge!(
        description: "Pool Size",
        unit: metrics::Unit::Count,
        "db.client.connection.idle.max",
        "db.client.connection.pool.name" => pool_name
    );

    let conn_get = gauge!(
        description: "Connection aquisition duration",
        unit: metrics::Unit::Seconds,
        "db.client.connection.create_time",
        "db.client.connection.pool.name" => pool_name
    );

    conn_count.set(active_connections as f64);
    idle_count.set(dbstate.idle_connections as f64);
    max_conn.set(pool_size as f64);
    conn_get.set(stats.get_wait_time.as_secs_f64());
}

#[allow(dead_code)]
struct DbConfig {
    name: Arc<str>,
    system: Arc<str>,
}
