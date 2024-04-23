use diesel::{RunQueryDsl, SelectableHelper};

use crate::database::{
    self,
    models::reminders::{CreateReminderData, Reminder},
    schema::reminders,
    DatabaseResult,
};

pub fn create_reminder(reminder: CreateReminderData) -> DatabaseResult<Reminder> {
    database::within_database(|conn| {
        let created_reminder = diesel::insert_into(reminders::table)
            .values(&reminder)
            .returning(Reminder::as_returning())
            .get_result(conn)?;

        Ok(created_reminder)
    })
}

#[cfg(test)]
mod tests {
    use self::database::init_database;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations};
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    use super::*;

    #[test]
    fn it_can_create_a_reminder() {
        init_database(":memory:", vec![MIGRATIONS]).unwrap();

        let reminder_data = CreateReminderData {
            date: "Testing".to_owned(),
            note: "Testing".to_owned(),
        };
        let reminder = create_reminder(reminder_data).unwrap();

        assert_eq!("Testing", reminder.note);
        assert_eq!("Testing", reminder.date);
    }
}
