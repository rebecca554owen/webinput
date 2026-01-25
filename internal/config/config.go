package config

import (
	"encoding/json"
	"os"
	"path/filepath"
)

// Config 应用程序配置结构体
type Config struct {
	Mode string `json:"mode"` // 运行模式："lan"（局域网）
	Port string `json:"port"` // 监听端口
	IP   string `json:"ip"`   // 监听IP地址
}

// DefaultConfig 默认配置
var DefaultConfig = Config{
	Mode: "lan",
	Port: "5000",
	IP:   "",
}

// getConfigPath 获取配置文件路径
func getConfigPath() string {
	var configDir string

	// 根据操作系统获取配置目录
	switch os.PathSeparator {
	case '\\':
		// Windows系统
		appData := os.Getenv("APPDATA")
		configDir = filepath.Join(appData, "WebInput")
	default:
		// macOS和Linux系统
		home := os.Getenv("HOME")
		configDir = filepath.Join(home, ".config", "webinput")
	}

	// 创建配置目录（如果不存在）
	os.MkdirAll(configDir, 0755)

	return filepath.Join(configDir, "config.json")
}

// LoadConfig 加载配置文件
func LoadConfig() Config {
	configPath := getConfigPath()

	// 检查配置文件是否存在
	if _, err := os.Stat(configPath); os.IsNotExist(err) {
		return DefaultConfig
	}

	// 读取配置文件
	data, err := os.ReadFile(configPath)
	if err != nil {
		return DefaultConfig
	}

	// 解析配置文件
	var config Config
	if err := json.Unmarshal(data, &config); err != nil {
		return DefaultConfig
	}

	// 确保配置值有效
	if config.Port == "" {
		config.Port = DefaultConfig.Port
	}
	if config.Mode == "" {
		config.Mode = DefaultConfig.Mode
	}

	return config
}

// Save 保存配置文件（实例方法）
func (c *Config) Save() error {
	return SaveConfig(*c)
}

// SaveConfig 保存配置文件
func SaveConfig(config Config) error {
	configPath := getConfigPath()

	// 序列化配置
	data, err := json.MarshalIndent(config, "", "  ")
	if err != nil {
		return err
	}

	// 写入配置文件
	return os.WriteFile(configPath, data, 0644)
}
