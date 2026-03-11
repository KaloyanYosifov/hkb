package components

import (
	"strings"

	"github.com/charmbracelet/lipgloss"
)

var (
	activeTabStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("15")).
		Background(lipgloss.Color("0")).
		Bold(true).
		Padding(0, 2)

	inactiveTabStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("8")).
		Background(lipgloss.Color("0")).
		Padding(0, 2)

	titleStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("12")).
		Background(lipgloss.Color("0")).
		Bold(true).
		Padding(0, 2)

	navBarStyle = lipgloss.NewStyle().
		Background(lipgloss.Color("0"))
)

type Navigation struct {
	tabs   []string
	active int
}

func NewNavigation(tabs []string) Navigation {
	return Navigation{tabs: tabs, active: 0}
}

func (n *Navigation) SetActive(idx int) {
	if idx >= 0 && idx < len(n.tabs) {
		n.active = idx
	}
}

func (n *Navigation) Active() int {
	return n.active
}

func (n Navigation) View(width int) string {
	title := titleStyle.Render("HKB")

	var tabParts []string
	for i, tab := range n.tabs {
		if i == n.active {
			tabParts = append(tabParts, activeTabStyle.Render(tab))
		} else {
			tabParts = append(tabParts, inactiveTabStyle.Render(tab))
		}
	}

	tabs := strings.Join(tabParts, "")
	bar := title + tabs

	return navBarStyle.Width(width).Render(bar)
}
