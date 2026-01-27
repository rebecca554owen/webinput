//go:build !windows

package build

import _ "embed"

//go:embed appicon.png
var trayIcon []byte
