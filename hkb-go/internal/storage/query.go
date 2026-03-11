package storage

import "github.com/kaloyan-yosifov/hkb-go/internal/dateutil"

type QueryOption interface {
	Apply(q *Query)
}

type Query struct {
	RemindAtGe      *dateutil.SimpleDate
	RemindAtLe      *dateutil.SimpleDate
	RemindAtBetween *[2]dateutil.SimpleDate
	WithIDs         []int64
	WithoutIDs      []int64
}

func BuildQuery(opts ...QueryOption) Query {
	var q Query
	for _, o := range opts {
		o.Apply(&q)
	}
	return q
}

type remindAtGeOption struct{ date dateutil.SimpleDate }

func (o remindAtGeOption) Apply(q *Query) { q.RemindAtGe = &o.date }
func RemindAtGe(d dateutil.SimpleDate) QueryOption { return remindAtGeOption{d} }

type remindAtLeOption struct{ date dateutil.SimpleDate }

func (o remindAtLeOption) Apply(q *Query) { q.RemindAtLe = &o.date }
func RemindAtLe(d dateutil.SimpleDate) QueryOption { return remindAtLeOption{d} }

type remindAtBetweenOption struct{ start, end dateutil.SimpleDate }

func (o remindAtBetweenOption) Apply(q *Query) {
	q.RemindAtBetween = &[2]dateutil.SimpleDate{o.start, o.end}
}
func RemindAtBetween(start, end dateutil.SimpleDate) QueryOption {
	return remindAtBetweenOption{start, end}
}

type withIDsOption struct{ ids []int64 }

func (o withIDsOption) Apply(q *Query) { q.WithIDs = o.ids }
func WithIDs(ids ...int64) QueryOption { return withIDsOption{ids} }

type withoutIDsOption struct{ ids []int64 }

func (o withoutIDsOption) Apply(q *Query) { q.WithoutIDs = o.ids }
func WithoutIDs(ids ...int64) QueryOption { return withoutIDsOption{ids} }
