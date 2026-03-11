package main

import (
	"fmt"
	"os"
	"strconv"
	"strings"

	"github.com/kaloyan-yosifov/hkb-go/internal/config"
	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
	"github.com/kaloyan-yosifov/hkb-go/internal/ipc"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage/sqlite"
)

const version = "0.1.0"

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	switch os.Args[1] {
	case "list":
		cmdList()
	case "create":
		cmdCreate()
	case "delete":
		cmdDelete()
	case "version":
		fmt.Printf("hkb %s\n", version)
	case "help", "-h", "--help":
		printUsage()
	default:
		fmt.Fprintf(os.Stderr, "unknown command: %s\n", os.Args[1])
		printUsage()
		os.Exit(1)
	}
}

func printUsage() {
	fmt.Println("Usage: hkb <command> [args]")
	fmt.Println()
	fmt.Println("Commands:")
	fmt.Println("  list                    List all reminders")
	fmt.Println("  create <note> <date>    Create a reminder (date is natural language)")
	fmt.Println("  delete <id>             Delete a reminder by ID")
	fmt.Println("  version                 Print version")
}

func openRepo() storage.Repository {
	if err := config.EnsureDataDir(); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	repo, err := sqlite.New(config.DBPath())
	if err != nil {
		fmt.Fprintf(os.Stderr, "error opening database: %v\n", err)
		os.Exit(1)
	}
	return repo
}

func notifyDaemon(event ipc.Event) {
	client, err := ipc.Connect()
	if err != nil {
		return
	}
	defer client.Close()
	_ = client.SendEvent(event)
}

func cmdList() {
	repo := openRepo()
	defer repo.Close()

	now := dateutil.Now()
	startOfDay := now.StartOfDay()
	endOfDay := now.EndOfDay()

	today, err := repo.FetchReminders(storage.RemindAtBetween(startOfDay, endOfDay))
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	upcoming, err := repo.FetchReminders(storage.RemindAtGe(endOfDay))
	if err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}

	if len(today) == 0 && len(upcoming) == 0 {
		fmt.Println("No reminders.")
		return
	}

	if len(today) > 0 {
		fmt.Println("Today:")
		for _, r := range today {
			printReminder(r, now)
		}
	}

	if len(upcoming) > 0 {
		if len(today) > 0 {
			fmt.Println()
		}
		fmt.Println("Upcoming:")
		for _, r := range upcoming {
			printReminder(r, now)
		}
	}
}

func printReminder(r storage.ReminderData, now dateutil.SimpleDate) {
	diff := r.RemindAt.Sub(now)
	humanDur := dateutil.HumanizeDuration(diff)
	dateStr := r.RemindAt.Format("02-01-2006 15:04")

	if humanDur == "" {
		fmt.Printf("  [%d] %s (%s)\n", r.ID, r.Note, dateStr)
	} else {
		fmt.Printf("  [%d] %s - %s (%s)\n", r.ID, r.Note, humanDur, dateStr)
	}
}

func cmdCreate() {
	if len(os.Args) < 4 {
		fmt.Fprintln(os.Stderr, "Usage: hkb create <note> <date>")
		fmt.Fprintln(os.Stderr, "  Example: hkb create \"Doctor appointment\" \"tomorrow at 15:00\"")
		os.Exit(1)
	}

	note := os.Args[2]
	dateStr := strings.Join(os.Args[3:], " ")

	parser := dateutil.NewHumanDateParser(dateutil.Now())
	parsedDate, err := parser.Parse(dateStr)
	if err != nil {
		fmt.Fprintf(os.Stderr, "invalid date: %v\n", err)
		os.Exit(1)
	}

	repo := openRepo()
	defer repo.Close()

	created, err := repo.CreateReminder(storage.CreateReminderData{
		Note:     note,
		RemindAt: parsedDate,
	})
	if err != nil {
		fmt.Fprintf(os.Stderr, "error creating reminder: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Created reminder [%d]: %s at %s\n", created.ID, created.Note, created.RemindAt.Format("02-01-2006 15:04"))

	if ev, err := ipc.NewReminderCreatedEvent(created); err == nil {
		notifyDaemon(ev)
	}
}

func cmdDelete() {
	if len(os.Args) < 3 {
		fmt.Fprintln(os.Stderr, "Usage: hkb delete <id>")
		os.Exit(1)
	}

	id, err := strconv.ParseInt(os.Args[2], 10, 64)
	if err != nil {
		fmt.Fprintf(os.Stderr, "invalid ID: %v\n", err)
		os.Exit(1)
	}

	repo := openRepo()
	defer repo.Close()

	if err := repo.DeleteReminder(id); err != nil {
		fmt.Fprintf(os.Stderr, "error deleting reminder: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("Deleted reminder [%d]\n", id)

	if ev, err := ipc.NewReminderDeletedEvent(id); err == nil {
		notifyDaemon(ev)
	}
}
