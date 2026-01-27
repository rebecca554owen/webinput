//go:build windows

package build

import _ "embed"

//go:embed windows/icon.ico
var trayIcon []byte
