use diesel::{
    sql_types::Date as SqlDateType, ExpressionMethods, IntoSql, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use hkb_date::date::SimpleDate;
use log::debug;

use crate::database::{
    self,
    models::reminders::{CreateReminder, Reminder, UpdateReminder},
    schema::reminders::{self, dsl as reminders_dsl},
    DatabaseResult,
};

#[derive(Debug)]
pub struct CreateReminderData {
    pub note: String,
    pub remind_at: SimpleDate,
}

#[derive(Debug)]
pub struct UpdateReminderData {
    pub id: i64,
    pub note: Option<String>,
    pub remind_at: Option<SimpleDate>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ReminderData {
    pub id: i64,
    pub note: String,
    pub remind_at: SimpleDate,
    pub created_at: SimpleDate,
}

impl Into<ReminderData> for Reminder {
    fn into(self) -> ReminderData {
        ReminderData {
            id: self.id,
            note: self.note,
            remind_at: SimpleDate::parse_from_rfc3339(self.remind_at).unwrap(),
            created_at: SimpleDate::parse_from_rfc3339(self.created_at).unwrap(),
        }
    }
}

impl Into<Reminder> for ReminderData {
    fn into(self) -> Reminder {
        Reminder {
            id: self.id,
            note: self.note,
            remind_at: self.remind_at.to_string(),
            created_at: self.created_at.to_string(),
        }
    }
}

impl Into<CreateReminder> for CreateReminderData {
    fn into(self) -> CreateReminder {
        CreateReminder {
            note: self.note,
            remind_at: self.remind_at.to_string(),
            created_at: SimpleDate::local().to_string(),
        }
    }
}

impl Into<UpdateReminder> for UpdateReminderData {
    fn into(self) -> UpdateReminder {
        UpdateReminder {
            note: self.note,
            remind_at: self.remind_at.map(|date| date.to_string()),
        }
    }
}

pub enum FetchRemindersOption {
    RemindAtBetween {
        end_date: SimpleDate,
        start_date: SimpleDate,
    },
}

pub fn fetch_reminders(
    options: Option<Vec<FetchRemindersOption>>,
) -> DatabaseResult<Vec<ReminderData>> {
    database::within_database(|conn| {
        let mut query = reminders_dsl::reminders
            .select(Reminder::as_select())
            .order_by(reminders_dsl::id.asc())
            .into_boxed();

        if let Some(options) = options {
            for option in options {
                match option {
                    FetchRemindersOption::RemindAtBetween {
                        end_date,
                        start_date,
                    } => {
                        query = query.filter(reminders_dsl::remind_at.between(
                            start_date.to_string().into_sql::<SqlDateType>(),
                            end_date.to_string().into_sql::<SqlDateType>(),
                        ));
                    }
                }
            }
        }

        let reminders: Vec<ReminderData> = query
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
    use hkb_date::date::{Duration, SimpleDate};
    use serial_test::serial;
    pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

    use super::*;

    macro_rules! create_a_reminder {
        () => {{
            let date =
                SimpleDate::parse_from_str("2024-04-05 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
            let reminder_data = CreateReminderData {
                remind_at: date,
                note: "Testing".to_owned(),
            };

            create_reminder(reminder_data).unwrap()
        }};

        ($date:expr) => {{
            let reminder_data = CreateReminderData {
                remind_at: $date,
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
        assert_eq!(
            reminder.remind_at.to_string(),
            fetched_reminder.remind_at.to_string()
        );
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
        let fetched_reminders = fetch_reminders(None).unwrap();

        assert_eq!(reminders.len(), fetched_reminders.len());

        for i in 0..fetched_reminders.len() {
            let reminder = fetched_reminders.get(i).unwrap();
            let expected_reminder = reminders.get(i).unwrap();

            assert_eq!(expected_reminder, reminder);
        }
    }

    #[test]
    #[serial]
    fn it_can_fetch_reminders_in_between() {
        truncate_table!();

        let d1 = SimpleDate::parse_from_str("2024-03-11 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let d2 = SimpleDate::parse_from_str("2024-03-12 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let start_date =
            SimpleDate::parse_from_str("2024-03-01 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end_date =
            SimpleDate::parse_from_str("2024-04-01 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();

        let reminders = vec![
            create_a_reminder!(d1.clone()),
            create_a_reminder!(d2.clone()),
            create_a_reminder!(),
            create_a_reminder!(),
        ];
        let fetched_reminders =
            fetch_reminders(Some(vec![FetchRemindersOption::RemindAtBetween {
                end_date,
                start_date,
            }]))
            .unwrap();

        assert_eq!(2, fetched_reminders.len());

        assert_eq!(reminders.get(0).unwrap(), fetched_reminders.get(0).unwrap());
        assert_eq!(reminders.get(1).unwrap(), fetched_reminders.get(1).unwrap());

        let start_date =
            SimpleDate::parse_from_str("2024-04-01 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end_date =
            SimpleDate::parse_from_str("2024-05-01 08:00:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let fetched_reminders =
            fetch_reminders(Some(vec![FetchRemindersOption::RemindAtBetween {
                end_date,
                start_date,
            }]))
            .unwrap();

        assert_eq!(2, fetched_reminders.len());

        assert_eq!(reminders.get(2).unwrap(), fetched_reminders.get(0).unwrap());
        assert_eq!(reminders.get(3).unwrap(), fetched_reminders.get(1).unwrap());
    }

    #[test]
    #[serial]
    fn it_can_create_a_reminder() {
        let date = SimpleDate::local();
        let reminder_data = CreateReminderData {
            remind_at: date,
            note: "Testing".to_owned(),
        };
        let reminder = create_reminder(reminder_data).unwrap();

        assert_eq!("Testing", reminder.note);
        assert_eq!(date.to_string(), reminder.remind_at.to_string());
    }

    #[test]
    #[serial]
    fn it_can_update_a_reminder() {
        let reminder = create_a_reminder!();
        let updated_reminder = update_reminder(UpdateReminderData {
            id: reminder.id,
            note: Some("Testing a new".to_owned()),
            remind_at: None,
        })
        .unwrap();

        assert_eq!("Testing a new", updated_reminder.note);
        assert_ne!(reminder.note, updated_reminder.note);
        assert_eq!(
            reminder.remind_at.to_string(),
            updated_reminder.remind_at.to_string()
        );
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
            remind_at: Some(date),
        })
        .unwrap();

        assert_eq!(reminder.note, updated_reminder.note);
        assert_ne!(reminder.remind_at.to_string(), expected_date);
        assert_eq!(expected_date, updated_reminder.remind_at.to_string());
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
