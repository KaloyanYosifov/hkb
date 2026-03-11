package util

type BoundedValue struct {
	val    int
	minVal int
	maxVal int
}

func NewBoundedValue(min, max int) BoundedValue {
	return BoundedValue{val: min, minVal: min, maxVal: max}
}

func (b *BoundedValue) Val() int    { return b.val }
func (b *BoundedValue) MaxVal() int { return b.maxVal }

func (b *BoundedValue) SetVal(v int) {
	if v > b.maxVal {
		v = b.maxVal
	}
	if v < b.minVal {
		v = b.minVal
	}
	b.val = v
}

func (b *BoundedValue) Add(v int) {
	b.SetVal(b.val + v)
}

func (b *BoundedValue) Sub(v int) {
	result := b.val - v
	if result < b.minVal {
		result = b.minVal
	}
	b.val = result
}

func (b *BoundedValue) SetMax(max int) {
	b.maxVal = max
	if b.val > max {
		b.val = max
	}
}

func (b *BoundedValue) SetToMin() { b.val = b.minVal }
func (b *BoundedValue) SetToMax() { b.val = b.maxVal }
