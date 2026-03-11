package components

import (
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

var (
	buttonFocusedStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("0")).
		Background(lipgloss.Color("15")).
		Padding(0, 3)

	buttonBlurredStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("15")).
		Background(lipgloss.Color("8")).
		Padding(0, 3)
)

type ButtonState struct {
	Label   string
	Focused bool
	Clicked bool
}

func NewButtonState(label string) ButtonState {
	return ButtonState{Label: label}
}

func (b *ButtonState) Focus()         { b.Focused = true }
func (b *ButtonState) Blur()          { b.Focused = false }
func (b *ButtonState) IsFocused() bool { return b.Focused }

func (b *ButtonState) Update(msg tea.Msg) tea.Cmd {
	if !b.Focused {
		return nil
	}
	if keyMsg, ok := msg.(tea.KeyMsg); ok {
		if keyMsg.String() == "enter" {
			b.Clicked = true
		}
	}
	return nil
}

func (b *ButtonState) WasClicked() bool {
	if b.Clicked {
		b.Clicked = false
		return true
	}
	return false
}

func (b ButtonState) View() string {
	style := buttonBlurredStyle
	if b.Focused {
		style = buttonFocusedStyle
	}
	return style.Render(b.Label)
}
