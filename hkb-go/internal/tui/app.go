package tui

import (
	"github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"

	"github.com/kaloyan-yosifov/hkb-go/internal/ipc"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
	"github.com/kaloyan-yosifov/hkb-go/internal/tui/components"
	"github.com/kaloyan-yosifov/hkb-go/internal/tui/views"
)

type AppView int

const (
	ViewMain AppView = iota
	ViewReminders
)

type Model struct {
	currentView      AppView
	editing          bool
	ignoreNavigation bool
	width            int
	height           int

	nav          components.Navigation
	mainView     views.MainModel
	remindersView views.RemindersModel
}

func NewModel(repo storage.Repository, daemonCh chan<- ipc.Event) Model {
	return Model{
		currentView:   ViewMain,
		nav:           components.NewNavigation([]string{"Main", "Reminders"}),
		mainView:      views.NewMainModel(),
		remindersView: views.NewRemindersModel(repo, daemonCh),
	}
}

func (m Model) Init() tea.Cmd {
	return nil
}

func (m Model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "ctrl+c":
			return m, tea.Quit
		case "esc":
			if m.editing {
				m.editing = false
				m.ignoreNavigation = false
			}
		case "tab":
			if !m.ignoreNavigation {
				if m.currentView == ViewMain {
					m.currentView = ViewReminders
				} else {
					m.currentView = ViewMain
				}
				m.nav.SetActive(int(m.currentView))
			}
		}

	case tea.WindowSizeMsg:
		m.width = msg.Width
		m.height = msg.Height
	}

	switch m.currentView {
	case ViewMain:
		var cmd tea.Cmd
		m.mainView, cmd = m.mainView.Update(msg)
		return m, cmd
	case ViewReminders:
		var cmd tea.Cmd
		m.remindersView, cmd = m.remindersView.Update(msg)
		m.editing = m.remindersView.IsEditing()
		m.ignoreNavigation = m.remindersView.IgnoreNavigation()
		return m, cmd
	}

	return m, nil
}

var (
	navStyle = lipgloss.NewStyle().
		Background(lipgloss.Color("0")).
		Foreground(lipgloss.Color("15")).
		Width(80)

	statusStyle = lipgloss.NewStyle().
		Background(lipgloss.Color("0")).
		Foreground(lipgloss.Color("15"))
)

func (m Model) View() string {
	navBar := m.nav.View(m.width)

	contentHeight := m.height - 2
	if contentHeight < 0 {
		contentHeight = 0
	}

	var content string
	switch m.currentView {
	case ViewMain:
		content = m.mainView.View(m.width, contentHeight)
	case ViewReminders:
		content = m.remindersView.View(m.width, contentHeight)
	}

	mode := "Normal Mode"
	if m.editing {
		mode = "Insert Mode"
	}
	statusBar := statusStyle.Width(m.width).Render(" " + mode)

	return navBar + "\n" + content + "\n" + statusBar
}
