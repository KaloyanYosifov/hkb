package storage

type Repository interface {
	FetchReminders(opts ...QueryOption) ([]ReminderData, error)
	FetchReminder(id int64) (ReminderData, error)
	CreateReminder(data CreateReminderData) (ReminderData, error)
	UpdateReminder(data UpdateReminderData) (ReminderData, error)
	DeleteReminder(id int64) error
	DeleteReminders(opts ...QueryOption) error
	Close() error
}
