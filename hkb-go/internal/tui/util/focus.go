package util

type Focusable interface {
	Focus()
	Blur()
	IsFocused() bool
}
