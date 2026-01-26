// +build darwin

package virtualkeyboard

import (
	"time"

	"github.com/micmonay/keybd_event"
)

// SendCtrlV macOS 特定实现：发送 Cmd+V 组合键
// macOS 上使用 Command 键而不是 Ctrl 键进行粘贴
func SendCtrlV() error {
	kb, err := keybd_event.NewKeyBonding()
	if err != nil {
		return err
	}

	kb.SetKeys(keybd_event.VK_V)
	kb.HasSuper(true) // Command 键在 keybd_event 中使用 Super 标志

	if err := kb.Press(); err != nil {
		return err
	}

	time.Sleep(20 * time.Millisecond)

	if err := kb.Release(); err != nil {
		return err
	}

	return nil
}

// SendCmdV SendCtrlV 的别名，保持 API 一致性
func SendCmdV() error {
	return SendCtrlV()
}
