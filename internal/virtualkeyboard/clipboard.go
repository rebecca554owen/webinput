package virtualkeyboard

import (
	"github.com/atotto/clipboard"
)

func CopyToClipboard(text string) error {
	return clipboard.WriteAll(text)
}

func PasteText(text string) error {
	if err := CopyToClipboard(text); err != nil {
		return err
	}

	if err := PasteFromClipboard(); err != nil {
		return err
	}

	return nil
}
