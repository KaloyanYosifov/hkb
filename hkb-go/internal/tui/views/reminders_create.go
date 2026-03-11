package views

import (
	"strings"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"

	"github.com/kaloyan-yosifov/hkb-go/internal/dateutil"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
	"github.com/kaloyan-yosifov/hkb-go/internal/tui/components"
)

var (
	formTitleStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("12")).
		Bold(true).
		Align(lipgloss.Center)

	errorStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("9")).
		Bold(true)
)

type RemindersCreateModel struct {
	titleInput components.InputState
	dateInput  components.InputState
	submitBtn  components.ButtonState
	focusIdx   int
	errorMsg   string

	submitted *storage.CreateReminderData
	cancelled bool
}

func NewRemindersCreateModel() RemindersCreateModel {
	m := RemindersCreateModel{
		titleInput: components.NewInputState("Title"),
		dateInput:  components.NewInputState("Reminder Date"),
		submitBtn:  components.NewButtonState("Submit"),
	}
	m.titleInput.Focus()
	m.titleInput.EditMode = true
	return m
}

func (m RemindersCreateModel) Update(msg tea.Msg) (RemindersCreateModel, tea.Cmd) {
	keyMsg, ok := msg.(tea.KeyMsg)
	if !ok {
		return m, nil
	}

	switch keyMsg.String() {
	case "esc":
		if m.focusIdx < 2 {
			input := m.activeInput()
			if input != nil && input.EditMode {
				input.EditMode = false
				return m, nil
			}
		}
		m.cancelled = true
		return m, nil
	case "tab":
		m.cycleFocus(1)
		return m, nil
	case "shift+tab":
		m.cycleFocus(-1)
		return m, nil
	}

	switch m.focusIdx {
	case 0:
		m.titleInput.Update(msg)
	case 1:
		m.dateInput.Update(msg)
	case 2:
		m.submitBtn.Update(msg)
		if m.submitBtn.WasClicked() {
			m.trySubmit()
		}
	}

	return m, nil
}

func (m *RemindersCreateModel) activeInput() *components.InputState {
	switch m.focusIdx {
	case 0:
		return &m.titleInput
	case 1:
		return &m.dateInput
	}
	return nil
}

func (m *RemindersCreateModel) cycleFocus(dir int) {
	m.blurAll()
	m.focusIdx = (m.focusIdx + dir + 3) % 3
	switch m.focusIdx {
	case 0:
		m.titleInput.Focus()
		m.titleInput.EditMode = true
	case 1:
		m.dateInput.Focus()
		m.dateInput.EditMode = true
	case 2:
		m.submitBtn.Focus()
	}
}

func (m *RemindersCreateModel) blurAll() {
	m.titleInput.Blur()
	m.titleInput.EditMode = false
	m.dateInput.Blur()
	m.dateInput.EditMode = false
	m.submitBtn.Blur()
}

func (m *RemindersCreateModel) trySubmit() {
	title := strings.TrimSpace(m.titleInput.Value())
	dateStr := strings.TrimSpace(m.dateInput.Value())

	if title == "" {
		m.errorMsg = "Title cannot be empty"
		return
	}
	if dateStr == "" {
		m.errorMsg = "Reminder date cannot be empty"
		return
	}

	parser := dateutil.NewHumanDateParser(dateutil.Now())
	parsedDate, err := parser.Parse(dateStr)
	if err != nil {
		m.errorMsg = "Invalid date: " + err.Error()
		return
	}

	m.errorMsg = ""
	data := storage.CreateReminderData{
		Note:     title,
		RemindAt: parsedDate,
	}
	m.submitted = &data
}

func (m RemindersCreateModel) View(width, height int) string {
	var b strings.Builder

	padding := height / 4
	for i := 0; i < padding; i++ {
		b.WriteString("\n")
	}

	formWidth := width * 60 / 100
	if formWidth < 40 {
		formWidth = 40
	}
	leftPad := (width - formWidth) / 2
	indent := strings.Repeat(" ", leftPad)

	b.WriteString(indent)
	b.WriteString(formTitleStyle.Width(formWidth).Render("Create Reminder"))
	b.WriteString("\n\n")

	b.WriteString(indent)
	b.WriteString(m.titleInput.View(formWidth))
	b.WriteString("\n\n")

	b.WriteString(indent)
	b.WriteString(m.dateInput.View(formWidth))
	b.WriteString("\n\n")

	if m.errorMsg != "" {
		b.WriteString(indent)
		b.WriteString(errorStyle.Render(m.errorMsg))
		b.WriteString("\n\n")
	}

	b.WriteString(indent)
	b.WriteString(m.submitBtn.View())
	b.WriteString("\n\n")

	b.WriteString(indent)
	b.WriteString(lipgloss.NewStyle().Foreground(lipgloss.Color("8")).Render("Tab/Shift+Tab: cycle fields  Esc: cancel"))

	return b.String()
}
