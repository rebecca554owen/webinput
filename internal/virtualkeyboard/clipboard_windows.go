//go:build windows
// +build windows

package virtualkeyboard

import "time"

func PasteFromClipboard() error {
	time.Sleep(50 * time.Millisecond)

	if err := SendCtrlV(); err != nil {
		return err
	}

	return nil
}
