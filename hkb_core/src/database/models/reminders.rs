use diesel::prelude::{Queryable, Selectable};

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::database::schema::reminders)]
#[cfg_attr(
    feature = "postgres-database",
    diesel(check_for_backend(diesel::pg::Pg))
)]
#[cfg_attr(
    feature = "mysql-database",
    diesel(check_for_backend(diesel::mysql::Mysql))
)]
#[cfg_attr(
    feature = "sqlite-database",
    diesel(check_for_backend(diesel::sqlite::Sqlite))
)]
pub struct Reminders {
    pub id: i64,
    pub date: String,
    pub note: String,
}
