use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use hkb_date::date::{Date, SimpleLocalDate};

use crate::database::{
    self,
    models::reminders::{CreateReminder, Reminder},
    schema::reminders,
    schema::reminders::dsl as reminders_dsl,
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

fn create_dto_from_model(reminder: Reminder) -> ReminderData {
    ReminderData {
        id: reminder.id,
        note: reminder.note,
        date: SimpleLocalDate::parse_from_str(reminder.date, "%Y-%m-%d %H:%M:%S%.3f %z").unwrap(),
    }
}

pub fn fetch_reminder(id: i64) -> DatabaseResult<ReminderData> {
    database::within_database(|conn| {
        let reminder = reminders_dsl::reminders
            .find(id)
            .select(Reminder::as_select())
            .first(conn)?;

        Ok(create_dto_from_model(reminder))
    })
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

        Ok(create_dto_from_model(created_reminder))
    })
}

#[cfg(test)]
mod tests {
    use self::database::init_database;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations};
    use hkb_date::date::SimpleLocalDate;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    use super::*;

    macro_rules! create_a_reminder {
        () => {{
            let date = SimpleLocalDate::now();
            let reminder_data = CreateReminderData {
                date,
                note: "Testing".to_owned(),
            };

            create_reminder(reminder_data).unwrap()
        }};
    }

    #[test]
    fn it_can_create_a_reminder() {
        init_database(":memory:", vec![MIGRATIONS]).unwrap();

        let date = SimpleLocalDate::now();
        let reminder_data = CreateReminderData {
            date,
            note: "Testing".to_owned(),
        };
        let reminder = create_reminder(reminder_data).unwrap();

        assert_eq!("Testing", reminder.note);
        assert_eq!(date.to_string(), reminder.date.to_string());
    }

    #[test]
    fn it_can_fetch_a_reminder() {
        init_database(":memory:", vec![MIGRATIONS]).unwrap();

        let reminder = create_a_reminder!();
        let fetched_reminder = fetch_reminder(reminder.id).unwrap();

        assert_eq!(reminder.id, fetched_reminder.id);
        assert_eq!(reminder.note, fetched_reminder.note);
        assert_eq!(reminder.date.to_string(), fetched_reminder.date.to_string());
    }
}
