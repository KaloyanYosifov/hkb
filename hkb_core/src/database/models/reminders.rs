use crate::database::schema::reminders;
use diesel::prelude::{Insertable, Queryable, Selectable};

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
pub struct Reminder {
    pub id: i64,
    pub date: String,
    pub note: String,
}

#[derive(Insertable)]
#[diesel(table_name = reminders)]
pub struct CreateReminderData {
    pub date: String,
    pub note: String,
}
