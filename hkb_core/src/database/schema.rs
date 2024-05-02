diesel::table! {
    reminders (id) {
        id -> Int8,
        note -> Varchar,
        date -> Date,
    }
}
