package virtualkeyboard

// SendText 发送文本（通过模拟键盘输入）
func SendText(text string) error {
	// 对于大多数现代应用程序，使用粘贴方法更可靠
	return PasteText(text)
}
