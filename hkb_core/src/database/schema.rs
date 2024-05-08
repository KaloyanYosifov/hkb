diesel::table! {
    reminders (id) {
        id -> Int8,
        note -> Varchar,
        remind_at -> Date,
        created_at -> Date,
    }
}
