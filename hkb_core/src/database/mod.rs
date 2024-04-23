use cfg_if::cfg_if;
use diesel::{migration, result::Error as DieselResultError, Connection, ConnectionError};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use log::debug;
use parking_lot::Mutex;
use thiserror::Error as ThisError;

pub mod models;
mod schema;
pub mod services;

#[derive(ThisError, Debug)]
pub enum DatabaseError {
    #[error("Database not initialized!")]
    DatabaseNotInitialized,
    #[error("Failed to run migrations")]
    FailedToRunMigrations,
    #[error("Failed to establish a connection!")]
    FailedToEstablishConnection(#[from] ConnectionError),
    #[error(transparent)]
    FailedToFetchResult(#[from] DieselResultError),
}

cfg_if! {
     if #[cfg(feature = "mysql-database" )] {
        use diesel::MysqlConnection;

        type DatabaseConnection = MysqlConnection;
    } else if #[cfg(feature = "sqlite-database" )] {
        use diesel::SqliteConnection;

        type DatabaseConnection = SqliteConnection;
    }
}

static GLOBAL_CONNECTION: Mutex<Option<DatabaseConnection>> = parking_lot::const_mutex(None);

pub type DatabaseResult<T> = Result<T, DatabaseError>;

pub fn init_database(url: &str, migrations: Vec<EmbeddedMigrations>) -> Result<(), DatabaseError> {
    let mut connection = {
        cfg_if! {
            if #[cfg(feature = "mysql-database")]  {
                MysqlConnection::establish(url)
            } else if #[cfg(feature = "sqlite-database")]  {
                SqliteConnection::establish(url)
            }
        }
    }?;

    debug!("Running migrations");
    // TODO: maybe we can use iter.enumurate() for this?
    // for now we just use a variable as it is easy
    let mut i = 1;
    for migration in migrations {
        debug!("Starting migration {i}.");
        match connection.run_pending_migrations(migration) {
            Err(e) => {
                println!("{e:?}");
                return Err(DatabaseError::FailedToRunMigrations);
            }
            _ => {}
        };
        debug!("Migration {i} finished !");

        i += 1;
    }

    let mut conn = GLOBAL_CONNECTION.lock();
    *conn = Some(connection);

    Ok(())
}

pub fn within_database<T, F: FnOnce(&mut DatabaseConnection) -> DatabaseResult<T>>(
    callback: F,
) -> DatabaseResult<T> {
    let mut conn = GLOBAL_CONNECTION.lock();

    if let Some(connection) = (*conn).as_mut() {
        callback(connection)
    } else {
        Err(DatabaseError::DatabaseNotInitialized)
    }
}
