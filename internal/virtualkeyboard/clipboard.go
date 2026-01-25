package virtualkeyboard

import (
	"time"

	"github.com/atotto/clipboard"
)

// CopyToClipboard 复制文本到剪贴板
func CopyToClipboard(text string) error {
	return clipboard.WriteAll(text)
}

// PasteFromClipboard 从剪贴板粘贴文本（需要配合键盘事件）
func PasteFromClipboard() error {
	// 等待一小段时间确保剪贴板操作完成
	time.Sleep(50 * time.Millisecond)

	// 发送 Ctrl+V 组合键来粘贴（更通用）
	if err := SendCtrlV(); err != nil {
		return err
	}

	return nil
}

// PasteText 复制文本到剪贴板并粘贴
func PasteText(text string) error {
	if err := CopyToClipboard(text); err != nil {
		return err
	}

	if err := PasteFromClipboard(); err != nil {
		return err
	}

	return nil
}
