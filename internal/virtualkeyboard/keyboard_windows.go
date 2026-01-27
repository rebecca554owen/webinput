//go:build windows
// +build windows

package virtualkeyboard

import (
	"time"
	"unsafe"

	"golang.org/x/sys/windows"
)

var (
	user32           = windows.NewLazySystemDLL("user32.dll")
	procSendInput    = user32.NewProc("SendInput")
	vkControl uint16 = 0x11
	vkV       uint16 = 0x56
	vkShift   uint16 = 0x10
	vkInsert  uint16 = 0x2D
	inputKeyboard    = uint32(1)
	keyEventfKeyUp   = uint32(0x0002)
	keyEventfKeydown = uint32(0x0000)
)

type KEYBDINPUT struct {
	WVk         uint16
	WScan       uint16
	DwFlags     uint32
	Time        uint32
	DwExtraInfo uintptr
}

type INPUT struct {
	Type uint32
	Ki   KEYBDINPUT
	_    [8]byte
}

func sendInput(inputs []INPUT) uint32 {
	success, _, _ := procSendInput.Call(
		uintptr(len(inputs)),
		uintptr(unsafe.Pointer(&inputs[0])),
		unsafe.Sizeof(inputs[0]),
	)
	return uint32(success)
}

func SendCtrlV() error {
	inputs := []INPUT{
		{Type: inputKeyboard, Ki: KEYBDINPUT{WVk: vkControl, DwFlags: keyEventfKeydown}},
		{Type: inputKeyboard, Ki: KEYBDINPUT{WVk: vkV, DwFlags: keyEventfKeydown}},
		{Type: inputKeyboard, Ki: KEYBDINPUT{WVk: vkV, DwFlags: keyEventfKeyUp}},
		{Type: inputKeyboard, Ki: KEYBDINPUT{WVk: vkControl, DwFlags: keyEventfKeyUp}},
	}
	sendInput(inputs)
	time.Sleep(20 * time.Millisecond)
	return nil
}

func SendShiftInsert() error {
	inputs := []INPUT{
		{Type: inputKeyboard, Ki: KEYBDINPUT{WVk: vkShift, DwFlags: keyEventfKeydown}},
		{Type: inputKeyboard, Ki: KEYBDINPUT{WVk: vkInsert, DwFlags: keyEventfKeydown}},
		{Type: inputKeyboard, Ki: KEYBDINPUT{WVk: vkInsert, DwFlags: keyEventfKeyUp}},
		{Type: inputKeyboard, Ki: KEYBDINPUT{WVk: vkShift, DwFlags: keyEventfKeyUp}},
	}
	sendInput(inputs)
	time.Sleep(20 * time.Millisecond)
	return nil
}
