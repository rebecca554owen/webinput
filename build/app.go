package build

import (
	"context"
	"fmt"
	"net"

	"github.com/rebecca554owen/webinput/internal/config"
	"github.com/rebecca554owen/webinput/internal/logger"
	"github.com/rebecca554owen/webinput/internal/server"
	"github.com/wailsapp/wails/v2/pkg/runtime"
)

// App Wails 应用结构
type App struct {
	ctx          context.Context
	config        *config.Config
	server        *server.Server
	allIPs        []string
	selectedIP    string
	mainIP        string
	accessURL     string
	isRunning     bool
}

// NewApp 创建新的应用实例
func NewApp() *App {
	cfg := config.LoadConfig()
	return &App{
		config:    &cfg,
		isRunning: false,
	}
}

// OnStartup Wails 应用启动时调用
func (a *App) OnStartup(ctx context.Context) {
	a.ctx = ctx
	logger.Info("WebInput 应用启动")

	// 获取 IP 地址
	a.allIPs = a.getAllIPs()
	if len(a.allIPs) > 1 {
		a.mainIP = a.allIPs[1] // 跳过 "0.0.0.0"
		a.selectedIP = a.mainIP
	} else {
		a.mainIP = "127.0.0.1"
		a.selectedIP = "127.0.0.1"
	}

	// 自动启动服务器
	if err := a.StartServer(); err != nil {
		logger.Error("自动启动服务器失败: " + err.Error())
	}
}

// OnShutdown Wails 应用关闭时调用
func (a *App) OnShutdown(ctx context.Context) {
	logger.Info("WebInput 应用关闭")
	a.StopServer()
}

// OnDomReady DOM 准备好后调用
func (a *App) OnDomReady(ctx context.Context) {
	// 可选：在这里执行 DOM 相关操作
}

// getAllIPs 获取所有可用的本机IP地址
func (a *App) getAllIPs() []string {
	var ips []string

	// 获取所有网络接口地址
	addrs, err := net.InterfaceAddrs()
	if err != nil {
		logger.Error("获取网络接口地址失败: " + err.Error())
		ips = append(ips, "127.0.0.1")
		return ips
	}

	for _, addr := range addrs {
		ipNet, ok := addr.(*net.IPNet)
		if !ok || ipNet.IP.IsLoopback() || ipNet.IP.IsLinkLocalUnicast() || ipNet.IP.To4() == nil {
			continue
		}
		ip := ipNet.IP.String()
		ips = append(ips, ip)
	}

	// 如果没有找到任何IP地址，返回默认值
	if len(ips) == 0 {
		ips = append(ips, "127.0.0.1")
	}

	// IP 分类排序（与 Python 版本一致）
	// 优先级：192.168.x.x > 10.x.x.x > 其他 > 虚拟网卡
	var priority192, priority10, otherIPs, virtualIPs []string

	for _, ip := range ips {
		// 检查IP前缀（优先匹配更具体的地址）
		if len(ip) >= 7 && ip[:7] == "198.18." {
			// Clash 等代理工具虚拟网卡（最优先检测）
			virtualIPs = append(virtualIPs, ip)
		} else if len(ip) >= 8 && ip[:8] == "192.168." {
			// 192.168.x.x (家庭/办公网络)
			priority192 = append(priority192, ip)
		} else if len(ip) >= 3 && ip[:3] == "10." {
			// 10.x.x.x (企业网络)
			priority10 = append(priority10, ip)
		} else if len(ip) >= 4 && ip[:4] == "172." {
			// 检查是否是虚拟网卡
			parts := splitIP(ip)
			if len(parts) >= 2 {
				second := parseIntSafe(parts[1])
				// Docker: 172.17.x.x, 172.18.x.x
				// Windows 虚拟网卡: 172.16.x.x
				// 私有网络范围: 172.16-31.x.x
				if second >= 16 && second <= 31 {
					virtualIPs = append(virtualIPs, ip)
				} else {
					otherIPs = append(otherIPs, ip)
				}
			} else {
				otherIPs = append(otherIPs, ip)
			}
		} else {
			otherIPs = append(otherIPs, ip)
		}
	}

	// 重新组合：优先级从高到低
	result := priority192
	result = append(result, priority10...)
	result = append(result, otherIPs...)
	result = append(result, virtualIPs...)

	// 将主要 IP 移到对应分类的第一位（保持分类顺序）
	mainIP := a.getLocalIP()
	result = a.moveMainIPTop(result, mainIP)

	// 在最前面添加 0.0.0.0（监听所有网卡）
	result = append([]string{"0.0.0.0"}, result...)

	return result
}

// getLocalIP 获取本机的主要IP地址
func (a *App) getLocalIP() string {
	conn, err := net.Dial("udp", "8.8.8.8:80")
	if err != nil {
		logger.Error("获取本机IP地址失败: " + err.Error())
		return "127.0.0.1"
	}
	defer conn.Close()

	localAddr := conn.LocalAddr().(*net.UDPAddr)
	return localAddr.IP.String()
}

// splitIP 分割IP地址
func splitIP(ip string) []string {
	var parts []string
	current := ""
	for _, c := range ip {
		if c == '.' {
			parts = append(parts, current)
			current = ""
		} else {
			current += string(c)
		}
	}
	parts = append(parts, current)
	return parts
}

// parseIntSafe 安全解析整数
func parseIntSafe(s string) int {
	var result int
	for _, c := range s {
		if c >= '0' && c <= '9' {
			result = result*10 + int(c-'0')
		} else {
			break
		}
	}
	return result
}

// moveMainIPTop 将主要IP移到对应分类的第一位
func (a *App) moveMainIPTop(ips []string, mainIP string) []string {
	// 找到主要IP的位置并分类
	for i, ip := range ips {
		if ip == mainIP {
			// 移除该位置的IP
			ips = append(ips[:i], ips[i+1:]...)
			// 根据主要IP的类型，插入到对应分类的开头
			var insertPos int
			if len(mainIP) >= 8 && mainIP[:8] == "192.168." {
				insertPos = 0
			} else if len(mainIP) >= 3 && mainIP[:3] == "10." {
				insertPos = countPrefix(ips, "192.168.")
			} else {
				insertPos = countPrefix(ips, "192.168.") + countPrefix(ips, "10.")
			}
			// 插入主要IP
			result := make([]string, 0, len(ips)+1)
			result = append(result, ips[:insertPos]...)
			result = append(result, mainIP)
			result = append(result, ips[insertPos:]...)
			return result
		}
	}
	return ips
}

// countPrefix 统计以指定前缀开头的IP数量
func countPrefix(ips []string, prefix string) int {
	count := 0
	for _, ip := range ips {
		if len(ip) >= len(prefix) && ip[:len(prefix)] == prefix {
			count++
		} else {
			break
		}
	}
	return count
}

// StartServer 启动 HTTP 服务器
func (a *App) StartServer() error {
	if a.isRunning {
		logger.Info("服务器已在运行中")
		return nil
	}

	// 创建并启动服务器实例
	a.server = server.NewServer(a.config)
	if err := a.server.Start(); err != nil {
		logger.Error("服务器启动失败: " + err.Error())
		return err
	}

	// 更新访问地址和运行状态
	a.accessURL = fmt.Sprintf("http://%s:%s", a.selectedIP, a.config.Port)
	a.isRunning = true

	logger.Info("服务器启动成功，访问地址: " + a.accessURL)
	return nil
}

// StopServer 停止 HTTP 服务器
func (a *App) StopServer() error {
	if !a.isRunning {
		logger.Info("服务器未运行")
		return nil
	}

	if a.server != nil {
		a.server.Stop()
	}

	a.isRunning = false
	logger.Info("服务器已停止")
	return nil
}

// GetAccessURL 获取访问地址
func (a *App) GetAccessURL() string {
	return a.accessURL
}

// GetIPs 获取所有可用的IP地址
func (a *App) GetIPs() []string {
	return a.allIPs
}

// SetSelectedIP 设置选中的IP地址
func (a *App) SetSelectedIP(ip string) {
	a.selectedIP = ip
	a.accessURL = fmt.Sprintf("http://%s:%s", a.selectedIP, a.config.Port)
}

// GetSelectedIP 获取选中的IP地址
func (a *App) GetSelectedIP() string {
	return a.selectedIP
}

// GetMainIP 获取主要IP地址
func (a *App) GetMainIP() string {
	return a.mainIP
}

// GetPort 获取当前端口
func (a *App) GetPort() string {
	return a.config.Port
}

// SetPort 设置端口
func (a *App) SetPort(port string) error {
	// 验证端口是否有效
	for _, c := range port {
		if c < '0' || c > '9' {
			return fmt.Errorf("端口必须为数字")
		}
	}

	wasRunning := a.isRunning
	if a.isRunning {
		a.StopServer()
	}
	a.config.Port = port
	a.config.Save()

	// 如果之前是运行状态，重新启动
	if wasRunning {
		a.StartServer()
	}

	return nil
}

// IsRunning 检查服务器是否运行
func (a *App) IsRunning() bool {
	return a.isRunning
}

// OpenBrowser 打开浏览器
func (a *App) OpenBrowser(url string) {
	runtime.BrowserOpenURL(a.ctx, url)
}
