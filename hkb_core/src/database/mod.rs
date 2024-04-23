use cfg_if::cfg_if;
use diesel::Connection;
use parking_lot::Mutex;

pub mod models;
mod schema;
pub mod services;

cfg_if! {
    if #[cfg(feature = "postgres-database" )] {
        use diesel::PgConnection;

        type DatabaseConnection = PgConnection;
    } else if #[cfg(feature = "mysql-database" )] {
        use diesel::MysqlConnection;

        type DatabaseConnection = MysqlConnection;
    } else if #[cfg(feature = "sqlite-database" )] {
        use diesel::SqliteConnection;

        type DatabaseConnection = SqliteConnection;
    }
}

static GLOBAL_CONNECTION: Mutex<Option<DatabaseConnection>> = parking_lot::const_mutex(None);

pub fn init_database(url: &str) {
    let connection = {
        #[cfg(feature = "postgres-database")]
        {
            PgConnection::establish(url)
        }
        #[cfg(feature = "mysql-database")]
        {
            MysqlConnection::establish(url)
        }
        #[cfg(feature = "sqlite-database")]
        {
            SqliteConnection::establish(url)
        }
    }
    .unwrap();

    let mut conn = GLOBAL_CONNECTION.lock();

    *conn = Some(connection);
}

pub fn within_database<T: FnOnce(&DatabaseConnection)>(callback: T) {
    let conn = GLOBAL_CONNECTION.lock();

    if let Some(connection) = (*conn).as_ref() {
        callback(connection);
    }
}
