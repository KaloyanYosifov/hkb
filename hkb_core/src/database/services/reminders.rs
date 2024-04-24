use diesel::{RunQueryDsl, SelectableHelper};
use hkb_date::date::{Date, SimpleLocalDate};

use crate::database::{
    self,
    models::reminders::{CreateReminder, Reminder},
    schema::reminders,
    DatabaseResult,
};

#[derive(Debug)]
pub struct CreateReminderData {
    pub date: SimpleLocalDate,
    pub note: String,
}

#[derive(Debug)]
pub struct ReminderData {
    pub id: i64,
    pub date: SimpleLocalDate,
    pub note: String,
}

pub fn create_reminder(reminder: CreateReminderData) -> DatabaseResult<ReminderData> {
    database::within_database(|conn| {
        let create_reminder = CreateReminder {
            note: reminder.note,
            date: reminder.date.to_string(),
        };
        let created_reminder = diesel::insert_into(reminders::table)
            .values(&create_reminder)
            .returning(Reminder::as_returning())
            .get_result(conn)?;
        println!("{:?}", created_reminder.date);
        let reminder_data: ReminderData = ReminderData {
            id: created_reminder.id,
            note: created_reminder.note,
            date: SimpleLocalDate::parse_from_str(
                created_reminder.date,
                "%Y-%m-%d %H:%M:%S%.3f %z",
            )
            .unwrap(),
        };

        Ok(reminder_data)
    })
}

#[cfg(test)]
mod tests {
    use self::database::init_database;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations};
    use hkb_date::date::SimpleLocalDate;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    use super::*;

    #[test]
    fn it_can_create_a_reminder() {
        init_database(":memory:", vec![MIGRATIONS]).unwrap();
        let date = SimpleLocalDate::now();
        let reminder_data = CreateReminderData {
            date: date.clone(),
            note: "Testing".to_owned(),
        };
        let reminder = create_reminder(reminder_data).unwrap();

        assert_eq!("Testing", reminder.note);
        assert_eq!(date.to_string(), reminder.date.to_string());
    }
}
