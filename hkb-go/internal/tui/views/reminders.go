package views

import (
	tea "github.com/charmbracelet/bubbletea"

	"github.com/kaloyan-yosifov/hkb-go/internal/ipc"
	"github.com/kaloyan-yosifov/hkb-go/internal/storage"
)

type remindersSubView int

const (
	subViewList remindersSubView = iota
	subViewCreate
)

type RemindersModel struct {
	repo     storage.Repository
	daemonCh chan<- ipc.Event

	subView    remindersSubView
	listModel  RemindersListModel
	createModel RemindersCreateModel
}

func NewRemindersModel(repo storage.Repository, daemonCh chan<- ipc.Event) RemindersModel {
	return RemindersModel{
		repo:        repo,
		daemonCh:    daemonCh,
		subView:     subViewList,
		listModel:   NewRemindersListModel(repo),
		createModel: NewRemindersCreateModel(),
	}
}

func (m RemindersModel) IsEditing() bool {
	return m.subView == subViewCreate
}

func (m RemindersModel) IgnoreNavigation() bool {
	return m.subView == subViewCreate
}

func (m RemindersModel) Update(msg tea.Msg) (RemindersModel, tea.Cmd) {
	switch m.subView {
	case subViewList:
		var cmd tea.Cmd
		m.listModel, cmd = m.listModel.Update(msg)

		if m.listModel.wantCreate {
			m.listModel.wantCreate = false
			m.subView = subViewCreate
			m.createModel = NewRemindersCreateModel()
			return m, nil
		}

		if m.listModel.deleteID != nil {
			id := *m.listModel.deleteID
			m.listModel.deleteID = nil
			_ = m.repo.DeleteReminder(id)
			if m.daemonCh != nil {
				if ev, err := ipc.NewReminderDeletedEvent(id); err == nil {
					select {
					case m.daemonCh <- ev:
					default:
					}
				}
			}
			m.listModel = NewRemindersListModel(m.repo)
		}

		return m, cmd

	case subViewCreate:
		var cmd tea.Cmd
		m.createModel, cmd = m.createModel.Update(msg)

		if m.createModel.cancelled {
			m.subView = subViewList
			m.listModel = NewRemindersListModel(m.repo)
			return m, nil
		}

		if m.createModel.submitted != nil {
			data := *m.createModel.submitted
			m.createModel.submitted = nil
			created, err := m.repo.CreateReminder(data)
			if err == nil && m.daemonCh != nil {
				if ev, err := ipc.NewReminderCreatedEvent(created); err == nil {
					select {
					case m.daemonCh <- ev:
					default:
					}
				}
			}
			m.subView = subViewList
			m.listModel = NewRemindersListModel(m.repo)
		}

		return m, cmd
	}

	return m, nil
}

func (m RemindersModel) View(width, height int) string {
	switch m.subView {
	case subViewList:
		return m.listModel.View(width, height)
	case subViewCreate:
		return m.createModel.View(width, height)
	}
	return ""
}
