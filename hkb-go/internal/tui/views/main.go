package views

import (
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

var welcomeStyle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("12")).
	Bold(true).
	Align(lipgloss.Center)

var subtitleStyle = lipgloss.NewStyle().
	Foreground(lipgloss.Color("8")).
	Align(lipgloss.Center)

type MainModel struct{}

func NewMainModel() MainModel {
	return MainModel{}
}

func (m MainModel) Update(msg tea.Msg) (MainModel, tea.Cmd) {
	return m, nil
}

func (m MainModel) View(width, height int) string {
	title := welcomeStyle.Width(width).Render("Welcome to HKB")
	subtitle := subtitleStyle.Width(width).Render("Holy Kern Base -- Your Personal Productivity Tool")
	hint := subtitleStyle.Width(width).Render("Press Tab to navigate between views")

	padding := ""
	topPad := height / 3
	for i := 0; i < topPad; i++ {
		padding += "\n"
	}

	return padding + title + "\n\n" + subtitle + "\n\n" + hint
}
