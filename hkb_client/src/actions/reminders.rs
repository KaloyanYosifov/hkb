use hkb_date::date::SimpleLocalDate;

#[derive(Debug)]
struct ReminderData {
    message: String,
    date: SimpleLocalDate,
}

#[derive(Debug)]
struct CreateReminderData {
    message: String,
    date: SimpleLocalDate,
}

pub fn create_a_reminder(data: CreateReminderData) -> ReminderData {
    todo!("Implement");
}
