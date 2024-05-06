use diesel::{
    sql_types::Date as SqlDateType, ExpressionMethods, IntoSql, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use hkb_date::date::{Date, SimpleDate, Timezone};
use log::debug;

use crate::database::{
    self,
    models::reminders::{CreateReminder, Reminder, UpdateReminder},
    schema::reminders::{self, dsl as reminders_dsl},
    DatabaseResult,
};

#[derive(Debug)]
pub struct CreateReminderData {
    pub date: SimpleDate,
    pub note: String,
}

#[derive(Debug)]
pub struct UpdateReminderData {
    pub id: i64,
    pub note: Option<String>,
    pub date: Option<SimpleDate>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ReminderData {
    pub id: i64,
    pub date: SimpleDate,
    pub note: String,
}

impl Into<ReminderData> for Reminder {
    fn into(self) -> ReminderData {
        ReminderData {
            id: self.id,
            note: self.note,
            date: SimpleDate::parse_from_rfc3339(self.date).unwrap(),
        }
    }
}

impl Into<Reminder> for ReminderData {
    fn into(self) -> Reminder {
        Reminder {
            id: self.id,
            note: self.note,
            date: self.date.to_string(),
        }
    }
}

impl Into<CreateReminder> for CreateReminderData {
    fn into(self) -> CreateReminder {
        CreateReminder {
            note: self.note,
            date: self.date.to_string(),
        }
    }
}

impl Into<UpdateReminder> for UpdateReminderData {
    fn into(self) -> UpdateReminder {
        UpdateReminder {
            note: self.note,
            date: self.date.map(|date| date.to_string()),
        }
    }
}

enum FetchRemindersOption {
    Between {
        end_date: SimpleDate,
        start_date: SimpleDate,
    },
}

pub fn fetch_reminders() -> DatabaseResult<Vec<ReminderData>> {
    database::within_database(|conn| {
        let mut d1 = SimpleDate::local();
        let mut d2 = SimpleDate::local();

        d1.sub_duration(hkb_date::date::Duration::Hour(5));
        d2.add_duration(hkb_date::date::Duration::Hour(4));

        let reminders: Vec<ReminderData> = reminders_dsl::reminders
            .select(Reminder::as_select())
            .order_by(reminders_dsl::id.asc())
            .filter(reminders_dsl::date.between(
                d1.to_string().into_sql::<SqlDateType>(),
                d2.to_string().into_sql::<SqlDateType>(),
            ))
            .get_results(conn)?
            .into_iter()
            .map(|reminder| reminder.into())
            .collect();

        Ok(reminders.into())
    })
}

pub fn fetch_reminder(id: i64) -> DatabaseResult<ReminderData> {
    database::within_database(|conn| {
        debug!(target: "REMINDERS_SERVICE", "Fetching reminder with id {id}");

        let reminder = reminders_dsl::reminders
            .find(id)
            .select(Reminder::as_select())
            .first(conn)?;

        debug!(target: "REMINDERS_SERVICE", "Found reminder {reminder:?}");

        Ok(reminder.into())
    })
}

pub fn create_reminder(reminder: CreateReminderData) -> DatabaseResult<ReminderData> {
    database::within_database(|conn| {
        debug!(target: "REMINDERS_SERVICE", "Creating reminder: {reminder:?}");

        let create_reminder: CreateReminder = reminder.into();
        let created_reminder = diesel::insert_into(reminders::table)
            .values(&create_reminder)
            .returning(Reminder::as_returning())
            .get_result(conn)?;

        debug!(target: "REMINDERS_SERVICE", "Reminder created. ID is: : {}", created_reminder.id);

        Ok(created_reminder.into())
    })
}

pub fn update_reminder(reminder: UpdateReminderData) -> DatabaseResult<ReminderData> {
    database::within_database(|conn| {
        debug!(target: "REMINDERS_SERVICE", "Updating reminder: {reminder:?}");

        let id = reminder.id;
        let update_reminder: UpdateReminder = reminder.into();
        let updated_reminder = diesel::update(reminders_dsl::reminders.find(id))
            .set(&update_reminder)
            .returning(Reminder::as_returning())
            .get_result(conn)?;

        debug!(target: "REMINDERS_SERVICE", "Reminder {id} updated!");

        Ok(updated_reminder.into())
    })
}

pub fn delete_reminder(id: i64) -> DatabaseResult<()> {
    database::within_database(|conn| {
        debug!(target: "REMINDERS_SERVICE", "Deleting reminder: {id}");

        diesel::delete(reminders_dsl::reminders.find(id)).execute(conn)?;

        debug!(target: "REMINDERS_SERVICE", "Deleted Reminder: {id}");

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use self::database::{init_database, within_database};
    use ctor::ctor;
    use diesel::sql_query;
    use diesel_migrations::{embed_migrations, EmbeddedMigrations};
    use hkb_date::date::{Date, Duration, SimpleDate};
    use serial_test::serial;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    use super::*;

    macro_rules! create_a_reminder {
        () => {{
            let date = SimpleDate::local();
            let reminder_data = CreateReminderData {
                date,
                note: "Testing".to_owned(),
            };

            create_reminder(reminder_data).unwrap()
        }};
    }

    macro_rules! truncate_table {
        () => {
            within_database(|conn| {
                sql_query("DELETE from reminders where 1=1")
                    .execute(conn)
                    .unwrap();

                Ok(())
            })
            .unwrap();
        };
    }

    #[test]
    #[ctor]
    fn init() {
        init_database(":memory:", vec![MIGRATIONS]).unwrap();
    }

    #[test]
    #[serial]
    fn it_can_fetch_a_reminder() {
        let reminder = create_a_reminder!();
        let fetched_reminder = fetch_reminder(reminder.id).unwrap();

        assert_eq!(reminder.id, fetched_reminder.id);
        assert_eq!(reminder.note, fetched_reminder.note);
        assert_eq!(reminder.date.to_string(), fetched_reminder.date.to_string());
    }

    #[test]
    #[serial]
    fn it_can_fetch_reminders() {
        truncate_table!();

        let reminders = vec![
            create_a_reminder!(),
            create_a_reminder!(),
            create_a_reminder!(),
        ];
        let fetched_reminders = fetch_reminders().unwrap();

        assert_eq!(reminders.len(), fetched_reminders.len());

        for i in 0..fetched_reminders.len() {
            let reminder = fetched_reminders.get(i).unwrap();
            let expected_reminder = reminders.get(i).unwrap();

            assert_eq!(expected_reminder, reminder);
        }
    }

    #[test]
    #[serial]
    fn it_can_create_a_reminder() {
        let date = SimpleDate::local();
        let reminder_data = CreateReminderData {
            date,
            note: "Testing".to_owned(),
        };
        let reminder = create_reminder(reminder_data).unwrap();

        assert_eq!("Testing", reminder.note);
        assert_eq!(date.to_string(), reminder.date.to_string());
    }

    #[test]
    #[serial]
    fn it_can_update_a_reminder() {
        let reminder = create_a_reminder!();
        let updated_reminder = update_reminder(UpdateReminderData {
            id: reminder.id,
            note: Some("Testing a new".to_owned()),
            date: None,
        })
        .unwrap();

        assert_eq!("Testing a new", updated_reminder.note);
        assert_ne!(reminder.note, updated_reminder.note);
        assert_eq!(reminder.date.to_string(), updated_reminder.date.to_string());
    }

    #[test]
    #[serial]
    fn it_can_update_date_of_a_reminder() {
        let reminder = create_a_reminder!();
        let mut date = SimpleDate::local();
        date.add_duration(Duration::Month(1)).unwrap();

        let expected_date = date.to_string();
        let updated_reminder = update_reminder(UpdateReminderData {
            id: reminder.id,
            note: None,
            date: Some(date),
        })
        .unwrap();

        assert_eq!(reminder.note, updated_reminder.note);
        assert_ne!(reminder.date.to_string(), expected_date);
        assert_eq!(expected_date, updated_reminder.date.to_string());
    }

    #[test]
    #[serial]
    fn it_can_delete_a_reminder() {
        let reminder = create_a_reminder!();
        let reminder2 = create_a_reminder!();

        assert!(fetch_reminder(reminder.id).is_ok());
        assert!(delete_reminder(reminder.id).is_ok());
        assert!(fetch_reminder(reminder.id).is_err());
        assert!(fetch_reminder(reminder2.id).is_ok());
    }
}
