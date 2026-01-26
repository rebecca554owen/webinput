// +build windows

package virtualkeyboard

import (
	"time"

	"github.com/micmonay/keybd_event"
)

// SendCtrlV 发送 Ctrl+V 组合键（Windows 粘贴方式）
func SendCtrlV() error {
	kb, err := keybd_event.NewKeyBonding()
	if err != nil {
		return err
	}

	kb.SetKeys(keybd_event.VK_V)
	kb.HasCTRL(true)

	if err := kb.Press(); err != nil {
		return err
	}

	time.Sleep(20 * time.Millisecond)

	if err := kb.Release(); err != nil {
		return err
	}

	return nil
}

// SendShiftInsert 发送 Shift+Insert 组合键（Windows 粘贴方式）
// 用于终端等不支持 Ctrl+V 的场景
func SendShiftInsert() error {
	kb, err := keybd_event.NewKeyBonding()
	if err != nil {
		return err
	}

	kb.SetKeys(keybd_event.VK_INSERT)
	kb.HasSHIFT(true)

	if err := kb.Press(); err != nil {
		return err
	}

	time.Sleep(20 * time.Millisecond)

	if err := kb.Release(); err != nil {
		return err
	}

	return nil
}
