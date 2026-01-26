package main

import (
	"context"
	"embed"

	"github.com/rebecca554owen/webinput/build"
	"github.com/wailsapp/wails/v2"
	"github.com/wailsapp/wails/v2/pkg/options"
	"github.com/wailsapp/wails/v2/pkg/options/assetserver"
	"github.com/wailsapp/wails/v2/pkg/options/mac"
)

//go:embed all:web
var assets embed.FS

func main() {
	// 创建应用实例
	app := build.NewApp()

	// 创建 Wails 应用
	err := wails.Run(&options.App{
		Title:  "WebInput",
		Width:  540,
		Height: 960,
		AssetServer: &assetserver.Options{
			Assets: assets,
		},
		BackgroundColour: &options.RGBA{R: 27, G: 38, B: 54, A: 1},
		OnStartup:        app.OnStartup,
		OnShutdown:       app.OnShutdown,
		OnDomReady:       app.OnDomReady,
		OnBeforeClose:    func(ctx context.Context) (prevent bool) {
			return false
		},
		Bind: []interface{}{
			app,
		},
		// Mac 特定选项
		Mac: &mac.Options{
			TitleBar: &mac.TitleBar{
				TitlebarAppearsTransparent: true,
				HideTitle:                  false,
				HideTitleBar:               false,
			},
		},
	})

	if err != nil {
		println("Error:", err.Error())
	}
}
