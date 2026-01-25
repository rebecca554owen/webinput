package gui

import (
	"fmt"
	"net/url"
	"os/exec"
	"runtime"

	"github.com/getlantern/systray"
	"github.com/rebecca554owen/webinput/internal/config"
	"github.com/rebecca554owen/webinput/internal/logger"
)

// GUI 图形界面实例
type GUI struct {
	config     *config.Config
	accessURL  string
	quitChan   chan bool
}

// NewGUI 创建新的图形界面实例
func NewGUI(config *config.Config, accessURL string) *GUI {
	return &GUI{
		config:     config,
		accessURL:  accessURL,
		quitChan:   make(chan bool),
	}
}

// Run 运行图形界面（系统托盘）
func (g *GUI) Run() {
	systray.Run(g.onReady, g.onExit)
}

// onReady 系统托盘初始化
func (g *GUI) onReady() {
	// 设置托盘图标（使用内置图标）
	systray.SetIcon(getIcon())
	systray.SetTitle("WebInput")
	systray.SetTooltip("WebInput - 远程输入服务")

	// 打开浏览器菜单项
	openURL := systray.AddMenuItem("打开访问地址", "在浏览器中打开访问地址")
	go func() {
		for {
			select {
			case <-openURL.ClickedCh:
				g.openBrowser(g.accessURL)
			case <-g.quitChan:
				return
			}
		}
	}()

	// 分隔符
	systray.AddMenuItem("-", "")

	// 退出菜单项
	mQuit := systray.AddMenuItem("退出", "退出WebInput服务")
	go func() {
		for {
			select {
			case <-mQuit.ClickedCh:
				systray.Quit()
				return
			case <-g.quitChan:
				return
			}
		}
	}()

	logger.Info("图形界面已启动（系统托盘）")
}

// onExit 系统托盘退出时调用
func (g *GUI) onExit() {
	close(g.quitChan)
	logger.Info("图形界面已退出")
}

// getIcon 获取托盘图标（简单的内置图标）
func getIcon() []byte {
	// 简单的16x16 ICO格式图标
	return []byte{
		0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x10, 0x10,
		0x00, 0x00, 0x01, 0x00, 0x20, 0x00, 0x68, 0x04,
		0x00, 0x00, 0x16, 0x00, 0x00, 0x00, 0x28, 0x00,
		0x00, 0x00, 0x10, 0x00, 0x00, 0x00, 0x20, 0x00,
		0x00, 0x00, 0x01, 0x00, 0x20, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
		0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
	}
}

// openBrowser 打开浏览器
func (g *GUI) openBrowser(urlStr string) {
	// 解析URL
	u, err := url.Parse(urlStr)
	if err != nil {
		logger.Error("解析URL失败: " + err.Error())
		return
	}

	// 打开浏览器
	var cmd *exec.Cmd
	switch runtime.GOOS {
	case "windows":
		cmd = exec.Command("cmd", "/c", "start", u.String())
	case "darwin":
		cmd = exec.Command("open", u.String())
	default:
		cmd = exec.Command("xdg-open", u.String())
	}

	if err := cmd.Start(); err != nil {
		logger.Error("打开浏览器失败: " + err.Error())
	}
}

// ShowMessage 显示消息
func (g *GUI) ShowMessage(title, msg string) {
	logger.Info(fmt.Sprintf("%s: %s", title, msg))
}