package components

import (
	"strings"
	"unicode"

	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

var (
	inputFocusedStyle = lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("12")).
		Padding(0, 1)

	inputBlurredStyle = lipgloss.NewStyle().
		Border(lipgloss.RoundedBorder()).
		BorderForeground(lipgloss.Color("8")).
		Padding(0, 1)

	labelStyle = lipgloss.NewStyle().
		Foreground(lipgloss.Color("12")).
		Bold(true)
)

type InputState struct {
	Label     string
	Buffer    string
	Focused   bool
	EditMode  bool
	cursorPos int
}

func NewInputState(label string) InputState {
	return InputState{Label: label}
}

func (s *InputState) Focus()     { s.Focused = true }
func (s *InputState) Blur()      { s.Focused = false }
func (s *InputState) IsFocused() bool { return s.Focused }

func (s *InputState) Value() string { return s.Buffer }

func (s *InputState) Update(msg tea.Msg) tea.Cmd {
	if !s.Focused {
		return nil
	}

	keyMsg, ok := msg.(tea.KeyMsg)
	if !ok {
		return nil
	}

	if s.EditMode {
		return s.updateInsertMode(keyMsg)
	}
	return s.updateNormalMode(keyMsg)
}

func (s *InputState) updateNormalMode(msg tea.KeyMsg) tea.Cmd {
	switch msg.String() {
	case "h", "left":
		if s.cursorPos > 0 {
			s.cursorPos--
		}
	case "l", "right":
		if s.cursorPos < len(s.Buffer) {
			s.cursorPos++
		}
	case "^":
		s.cursorPos = 0
	case "$":
		s.cursorPos = len(s.Buffer)
	case "e":
		s.cursorPos = s.wordEndForward()
	case "b":
		s.cursorPos = s.wordStartBackward()
	case "i":
		s.EditMode = true
	case "I":
		s.cursorPos = 0
		s.EditMode = true
	case "a":
		if s.cursorPos < len(s.Buffer) {
			s.cursorPos++
		}
		s.EditMode = true
	case "A":
		s.cursorPos = len(s.Buffer)
		s.EditMode = true
	}
	return nil
}

func (s *InputState) insertChar(ch rune) {
	runes := []rune(s.Buffer)
	newRunes := make([]rune, 0, len(runes)+1)
	newRunes = append(newRunes, runes[:s.cursorPos]...)
	newRunes = append(newRunes, ch)
	newRunes = append(newRunes, runes[s.cursorPos:]...)
	s.Buffer = string(newRunes)
	s.cursorPos++
}

func (s *InputState) updateInsertMode(msg tea.KeyMsg) tea.Cmd {
	switch msg.Type {
	case tea.KeyRunes:
		runes := []rune(s.Buffer)
		newRunes := make([]rune, 0, len(runes)+len(msg.Runes))
		newRunes = append(newRunes, runes[:s.cursorPos]...)
		newRunes = append(newRunes, msg.Runes...)
		newRunes = append(newRunes, runes[s.cursorPos:]...)
		s.Buffer = string(newRunes)
		s.cursorPos += len(msg.Runes)
	case tea.KeySpace:
		s.insertChar(' ')
	case tea.KeyBackspace:
		if s.cursorPos > 0 {
			runes := []rune(s.Buffer)
			s.Buffer = string(append(runes[:s.cursorPos-1], runes[s.cursorPos:]...))
			s.cursorPos--
		}
	case tea.KeyLeft:
		if s.cursorPos > 0 {
			s.cursorPos--
		}
	case tea.KeyRight:
		if s.cursorPos < len(s.Buffer) {
			s.cursorPos++
		}
	}
	return nil
}

func charClass(r rune) int {
	if unicode.IsSpace(r) {
		return 0
	}
	if unicode.IsPunct(r) || unicode.IsSymbol(r) {
		return 1
	}
	return 2
}

func (s *InputState) wordEndForward() int {
	runes := []rune(s.Buffer)
	pos := s.cursorPos
	if pos >= len(runes) {
		return pos
	}
	pos++
	for pos < len(runes) && unicode.IsSpace(runes[pos]) {
		pos++
	}
	if pos >= len(runes) {
		return len(runes)
	}
	cls := charClass(runes[pos])
	for pos < len(runes) && charClass(runes[pos]) == cls {
		pos++
	}
	return pos
}

func (s *InputState) wordStartBackward() int {
	runes := []rune(s.Buffer)
	pos := s.cursorPos
	if pos <= 0 {
		return 0
	}
	pos--
	for pos > 0 && unicode.IsSpace(runes[pos]) {
		pos--
	}
	if pos <= 0 {
		return 0
	}
	cls := charClass(runes[pos])
	for pos > 0 && charClass(runes[pos-1]) == cls {
		pos--
	}
	return pos
}

func (s InputState) View(width int) string {
	label := labelStyle.Render(s.Label)

	display := s.Buffer
	if s.Focused && s.EditMode && s.cursorPos <= len(s.Buffer) {
		runes := []rune(display)
		var b strings.Builder
		for i, r := range runes {
			if i == s.cursorPos {
				b.WriteString(lipgloss.NewStyle().Reverse(true).Render(string(r)))
			} else {
				b.WriteRune(r)
			}
		}
		if s.cursorPos == len(runes) {
			b.WriteString(lipgloss.NewStyle().Reverse(true).Render(" "))
		}
		display = b.String()
	}

	style := inputBlurredStyle
	if s.Focused {
		style = inputFocusedStyle
	}

	inputBox := style.Width(width - 4).Render(display)
	return label + "\n" + inputBox
}
