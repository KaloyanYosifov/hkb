package ipc

import (
	"encoding/binary"
	"fmt"
)

const (
	FrameSize         = 16384
	FrameMetadataSize = 4
	FrameDataSize     = FrameSize - FrameMetadataSize
)

type Frame struct {
	Size          uint16
	FrameNumber   uint8
	RelatedFrames uint8
	Data          [FrameDataSize]byte
}

func (f *Frame) DataSlice() []byte {
	if int(f.Size) > FrameDataSize {
		return f.Data[:]
	}
	return f.Data[:f.Size]
}

func (f *Frame) ToBytes() [FrameSize]byte {
	var buf [FrameSize]byte
	binary.LittleEndian.PutUint16(buf[0:2], f.Size)
	buf[2] = f.FrameNumber
	buf[3] = f.RelatedFrames
	copy(buf[4:], f.Data[:])
	return buf
}

func FrameFromBytes(buf [FrameSize]byte) Frame {
	f := Frame{
		Size:          binary.LittleEndian.Uint16(buf[0:2]),
		FrameNumber:   buf[2],
		RelatedFrames: buf[3],
	}
	copy(f.Data[:], buf[4:])
	return f
}

func FramesFromEvent(event Event) ([]Frame, error) {
	data, err := MarshalEvent(event)
	if err != nil {
		return nil, fmt.Errorf("marshal event for framing: %w", err)
	}

	totalFrames := (len(data) + FrameDataSize - 1) / FrameDataSize
	if totalFrames > 255 {
		return nil, fmt.Errorf("event too large: %d bytes requires %d frames (max 255)", len(data), totalFrames)
	}

	frames := make([]Frame, 0, totalFrames)
	for i := 0; i < totalFrames; i++ {
		start := i * FrameDataSize
		end := start + FrameDataSize
		if end > len(data) {
			end = len(data)
		}
		chunk := data[start:end]

		f := Frame{
			Size:          uint16(len(chunk)),
			FrameNumber:   uint8(i + 1),
			RelatedFrames: uint8(totalFrames),
		}
		copy(f.Data[:], chunk)
		frames = append(frames, f)
	}

	return frames, nil
}

func AssembleFrames(frames []Frame) (Event, error) {
	var combined []byte
	for _, f := range frames {
		combined = append(combined, f.DataSlice()...)
	}
	return UnmarshalEvent(combined)
}
