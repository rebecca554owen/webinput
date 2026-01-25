package virtualkeyboard

import (
	"runtime"
	"time"

	"github.com/micmonay/keybd_event"
)

// SendCtrlV 发送 Ctrl+V 组合键（用于粘贴）
func SendCtrlV() error {
	// 初始化键盘事件
	kb, err := keybd_event.NewKeyBonding()
	if err != nil {
		return err
	}

	// 设置 V 键
	kb.SetKeys(keybd_event.VK_V)

	// 设置 Ctrl 键
	kb.HasCTRL(true)

	// 发送键盘事件
	if err := kb.Press(); err != nil {
		return err
	}

	// 等待一小段时间确保事件被处理
	time.Sleep(20 * time.Millisecond)

	// 释放键盘事件
	if err := kb.Release(); err != nil {
		return err
	}

	return nil
}

// SendShiftInsert 发送 Shift+Insert 组合键（用于粘贴）
func SendShiftInsert() error {
	// 初始化键盘事件
	kb, err := keybd_event.NewKeyBonding()
	if err != nil {
		return err
	}

	// 根据操作系统设置键盘布局
	if runtime.GOOS == "windows" {
		kb.SetKeys(keybd_event.VK_INSERT)
	} else if runtime.GOOS == "darwin" {
		kb.SetKeys(keybd_event.VK_INSERT)
	} else {
		// Linux系统可能需要使用不同的键码
		kb.SetKeys(keybd_event.VK_INSERT)
	}

	// 设置组合键标志
	kb.HasSHIFT(true)

	// 发送键盘事件
	if err := kb.Press(); err != nil {
		return err
	}

	// 等待一小段时间确保事件被处理
	time.Sleep(20 * time.Millisecond)

	// 释放键盘事件
	if err := kb.Release(); err != nil {
		return err
	}

	return nil
}

// SendText 发送文本（通过模拟键盘输入）
func SendText(text string) error {
	// 对于大多数现代应用程序，使用粘贴方法更可靠
	return PasteText(text)
}
