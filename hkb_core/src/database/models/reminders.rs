use crate::database::schema::reminders;
use diesel::{
    prelude::{Insertable, Queryable, Selectable},
    query_builder::AsChangeset,
};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = reminders)]
#[cfg_attr(
    feature = "mysql-database",
    diesel(check_for_backend(diesel::mysql::Mysql))
)]
#[cfg_attr(
    feature = "sqlite-database",
    diesel(check_for_backend(diesel::sqlite::Sqlite))
)]
pub(crate) struct Reminder {
    pub id: i64,
    pub date: String,
    pub note: String,
}

#[derive(Debug, AsChangeset)]
#[diesel(table_name = reminders)]
pub(crate) struct UpdateReminder {
    pub note: Option<String>,
    pub date: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = reminders)]
pub(crate) struct CreateReminder {
    pub date: String,
    pub note: String,
}
