# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WebInput 是一个基于 Wails 的桌面应用，允许手机通过浏览器远程输入到电脑。核心功能是通过 WebSocket 实现手机端和桌面端的实时双向通信，支持预览、同步和上屏。

## 架构概览

```
┌─────────────────┐     WebSocket      ┌─────────────────┐
│   桌面端应用      │ ←───────────────→ │   手机端浏览器    │
│  (Wails/Go)     │      Hub 模式       │   (Web/JS)      │
└─────────────────┘                    └─────────────────┘
        │                                         │
        │                                         │
        ▼                                         ▼
┌─────────────────┐                    ┌─────────────────┐
│  虚拟键盘输入     │                    │   实时预览界面    │
│  (Win32 API)    │                    │   (mobile.html) │
└─────────────────┘                    └─────────────────┘
```

### 核心组件

**Hub 架构** (`internal/websocket/hub.go`):
- 单一桌面端连接 (`desktop`)
- 多个手机端连接 (`mobiles` map)
- 设备会话管理 (`deviceSessions`) - 支持重连后恢复内容
- 消息路由：preview（预览）、sync（同步）、send（上屏）、clear（清空）

**嵌入资源**:
- `internal/server/assets/mobile.html` 通过 `//go:embed` 嵌入到二进制文件
- `web/` 目录通过 `//go:embed all:web` 嵌入到应用（main.go）

**消息协议** (`internal/websocket/types.go`):
```go
type WSMessage struct {
    Type      MessageType   // preview, sync, send, clear, connect
    Data      interface{}   // 消息数据
    Timestamp int64         // 时间戳
}
```

## 开发命令

### 构建和运行
```bash
# 开发模式（支持热重载）
wails dev

# 生产构建（单文件）
wails build --clean

# 构建产物位置
build/bin/webinput.exe  # Windows
```

### 前端开发
```bash
cd frontend
npm install      # 安装依赖
npm run dev      # 开发服务器（Vite）
npm run build    # 生产构建
```

### 版本管理
```bash
# 更新 wails.json 中的版本号
# 然后运行 GitHub Actions 发布
```

## 关键文件说明

| 文件 | 作用 |
|------|------|
| `main.go` | 应用入口，嵌入 web 资源，配置 Wails 选项 |
| `build/app.go` | Wails 应用逻辑，管理服务器生命周期和 IP 获取 |
| `internal/server/server.go` | HTTP 服务器，路由处理，嵌入 mobile.html |
| `internal/websocket/hub.go` | WebSocket Hub，管理所有连接和消息路由 |
| `internal/websocket/types.go` | WebSocket 消息类型定义 |
| `internal/virtualkeyboard/` | Windows 虚拟键盘实现（Win32 API） |
| `frontend/main.js` | 桌面端 WebSocket 客户端，UI 逻辑 |
| `internal/server/assets/mobile.html` | 手机端界面（嵌入到二进制） |

## WebSocket 消息流

```
手机输入 → preview → Hub → 桌面显示预览
桌面编辑 → sync → Hub → 广播到所有手机
手机确认 → send → Hub → 调用虚拟键盘上屏
桌面清空 → clear → Hub → 指定手机清空
```

## 多设备支持

- 每个手机设备有唯一 `device_id`（存储在 localStorage）
- Hub 维护 `deviceSessions` 用于跨重连恢复
- 桌面端为每个设备创建独立预览框
- 支持同时连接多个手机

## 发布流程

1. 更新 `wails.json` 中的 `productVersion`
2. 推送代码到 GitHub
3. 在 Actions 页面运行 "Release Windows Build"
4. 输入版本号（如 `v1.0.0`）
5. 自动构建并创建 Release

## 注意事项

- 修改 `mobile.html` 后需要重新编译才能生效（嵌入资源）
- Go 版本由 `go.mod` 中的 `go 1.25` 指定
- 单文件发布：所有资源嵌入到 `webinput.exe`，无需额外依赖
