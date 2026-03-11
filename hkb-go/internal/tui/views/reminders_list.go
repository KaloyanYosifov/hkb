package views

import (
	"fmt"
	"strings"
	"time"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"

	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
	"github.com/kaloyan-yosifov/hkb-go/internal/tui/util"
)

var (
	sectionTitleStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("12")).
		Bold(true).
		MarginBottom(1)

	selectedStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("15")).
		Background(lipgloss.Color("4"))

	normalItemStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("15"))

	hintStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("8")).
		Italic(true)
)

type RemindersListModel struct {
	today    []storage.ReminderData
	upcoming []storage.ReminderData
	selected util.BoundedValue

	wantCreate bool
	deleteID   *int64

	lastD    time.Time
	dPressed bool
}

func NewRemindersListModel(repo storage.Repository) RemindersListModel {
	now := dateutil.Now()
	startOfDay := now.StartOfDay()
	endOfDay := now.EndOfDay()

	today, _ := repo.FetchReminders(storage.RemindAtBetween(startOfDay, endOfDay))
	upcoming, _ := repo.FetchReminders(storage.RemindAtGe(endOfDay))

	total := len(today) + len(upcoming)
	maxIdx := 0
	if total > 0 {
		maxIdx = total - 1
	}

	return RemindersListModel{
		today:    today,
		upcoming: upcoming,
		selected: util.NewBoundedValue(0, maxIdx),
	}
}

func (m RemindersListModel) Update(msg tea.Msg) (RemindersListModel, tea.Cmd) {
	keyMsg, ok := msg.(tea.KeyMsg)
	if !ok {
		return m, nil
	}

	switch keyMsg.String() {
	case "j", "down":
		m.selected.Add(1)
	case "k", "up":
		m.selected.Sub(1)
	case "a", "A":
		m.wantCreate = true
	case "d":
		now := time.Now()
		if m.dPressed && now.Sub(m.lastD) < 300*time.Millisecond {
			m.tryDelete()
			m.dPressed = false
		} else {
			m.dPressed = true
			m.lastD = now
		}
	case "backspace":
		m.tryDelete()
	}

	return m, nil
}

func (m *RemindersListModel) tryDelete() {
	idx := m.selected.Val()
	if idx < len(m.today) {
		id := m.today[idx].ID
		m.deleteID = &id
	} else {
		upIdx := idx - len(m.today)
		if upIdx < len(m.upcoming) {
			id := m.upcoming[upIdx].ID
			m.deleteID = &id
		}
	}
}

func (m RemindersListModel) View(width, height int) string {
	var b strings.Builder

	todayHeight := height * 30 / 100
	upcomingHeight := height - todayHeight

	b.WriteString(sectionTitleStyle.Render("Today Reminders"))
	b.WriteString("\n")

	if len(m.today) == 0 {
		b.WriteString(hintStyle.Render("  No reminders for today"))
		b.WriteString("\n")
	} else {
		for i, r := range m.today {
			line := formatReminder(r)
			if i == m.selected.Val() {
				b.WriteString(selectedStyle.Width(width).Render(line))
			} else {
				b.WriteString(normalItemStyle.Render(line))
			}
			b.WriteString("\n")
		}
	}

	for b.Len() < todayHeight*width/2 {
		b.WriteString("\n")
	}

	b.WriteString(sectionTitleStyle.Render("Upcoming Reminders"))
	b.WriteString("\n")

	if len(m.upcoming) == 0 {
		b.WriteString(hintStyle.Render("  No upcoming reminders"))
		b.WriteString("\n")
	} else {
		for i, r := range m.upcoming {
			globalIdx := len(m.today) + i
			line := formatReminder(r)
			if globalIdx == m.selected.Val() {
				b.WriteString(selectedStyle.Width(width).Render(line))
			} else {
				b.WriteString(normalItemStyle.Render(line))
			}
			b.WriteString("\n")
		}
	}

	_ = upcomingHeight

	b.WriteString("\n")
	b.WriteString(hintStyle.Render("  j/k: navigate  a: create  dd: delete"))

	return b.String()
}

func formatReminder(r storage.ReminderData) string {
	now := dateutil.Now()
	diff := r.RemindAt.Sub(now)
	humanDur := dateutil.HumanizeDuration(diff)
	dateStr := r.RemindAt.Format("02-01-2006")

	if humanDur == "" {
		return fmt.Sprintf("  %s (%s)", r.Note, dateStr)
	}
	return fmt.Sprintf("  %s - %s (%s)", r.Note, humanDur, dateStr)
}
