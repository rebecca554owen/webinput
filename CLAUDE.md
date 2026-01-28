# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

WebInput 是一个基于 Tauri 的桌面应用，允许手机通过浏览器远程输入到电脑。核心功能是通过 WebSocket 实现手机端和桌面端的实时双向通信，支持预览、同步和上屏。

## 架构概览

```
┌─────────────────┐     WebSocket      ┌─────────────────┐
│   桌面端应用      │ ←───────────────→ │   手机端浏览器    │
│   (Tauri/Rust)  │      Hub 模式       │   (Web/JS)      │
└─────────────────┘                    └─────────────────┘
        │                                         │
        │                                         │
        ▼                                         ▼
┌─────────────────┐                    ┌─────────────────┐
│  虚拟键盘输入     │                    │   实时预览界面    │
│  (平台 API)     │                    │   (index.html)  │
└─────────────────┘                    └─────────────────┘
```

### 核心组件

**Hub 架构** (`src-tauri/src/hub.rs`):
- 单一桌面端连接 (`desktop`)
- 多个手机端连接 (`mobiles` HashMap)
- 设备会话管理 (`device_sessions`) - 支持重连后恢复内容
- 消息路由：preview（预览）、sync（同步）、send（上屏）、clear（清空）

**Tauri Commands** (`src-tauri/src/commands.rs`):
- `get_server_info()` - 获取服务器信息（IP、端口）
- `send_text_to_keyboard(text)` - 发送文本到虚拟键盘

**HTTP 服务器** (`src-tauri/src/server.rs`):
- Axum Web 框架
- 静态文件服务（index.html）
- WebSocket 升级处理

**消息协议** (`src-tauri/src/types.rs`):
```rust
pub struct WSMessage {
    pub msg_type: String,  // preview, sync, send, clear, connect
    pub data: serde_json::Value,
    pub timestamp: i64,
}
```

## 开发命令

### 构建和运行
```bash
# 开发模式（支持热重载）
npm run tauri:dev

# 生产构建
npm run tauri:build

# 构建产物位置
# Windows: src-tauri/target/release/bundle/nsis/
# macOS: src-tauri/target/release/bundle/dmg/
# Linux: src-tauri/target/release/bundle/appimage/
```

### 前端开发
```bash
npm install      # 安装依赖
npm run dev      # 开发服务器（Vite）
npm run build    # 生产构建到 dist/
```

### 版本管理
```bash
# 更新 src-tauri/Cargo.toml 和 src-tauri/tauri.conf.json 中的版本号
# 然后运行 GitHub Actions 发布
```

## 关键文件说明

| 文件 | 作用 |
|------|------|
| `src-tauri/src/main.rs` | Rust 应用入口 |
| `src-tauri/src/lib.rs` | Tauri 库入口，注册 commands |
| `src-tauri/src/commands.rs` | Tauri 命令定义（前后端通信） |
| `src-tauri/src/app.rs` | 应用生命周期管理 |
| `src-tauri/src/server.rs` | Axum HTTP 服务器 |
| `src-tauri/src/hub.rs` | WebSocket Hub，连接和消息路由 |
| `src-tauri/src/types.rs` | WebSocket 消息类型定义 |
| `src-tauri/src/virtual_keyboard.rs` | 虚拟键盘实现（Windows/macOS/Linux） |
| `src-tauri/src/config.rs` | 配置管理 |
| `src-tauri/Cargo.toml` | Rust 依赖配置 |
| `src-tauri/tauri.conf.json` | Tauri 应用配置 |
| `frontend/main.js` | 桌面端 WebSocket 客户端，UI 逻辑 |
| `index.html` | 手机端界面（通过 Tauri 嵌入） |

## WebSocket 消息流

```
手机输入 → preview → Hub → 桌面显示预览
桌面编辑 → sync → Hub → 广播到所有手机
手机确认 → send → Hub → 调用虚拟键盘上屏
桌面清空 → clear → Hub → 指定手机清空
```

## 多设备支持

- 每个手机设备有唯一 `device_id`（存储在 localStorage）
- Hub 维护 `device_sessions` 用于跨重连恢复
- 桌面端为每个设备创建独立预览框
- 支持同时连接多个手机

## 发布流程

1. 更新 `src-tauri/Cargo.toml` 和 `src-tauri/tauri.conf.json` 中的版本号
2. 推送代码到 GitHub
3. 在 Actions 页面运行 "Release Windows Build" 或 "Release Multi-Platform Build"
4. 输入版本号（如 `v1.0.0`）
5. 自动构建并创建 Release

## 技术栈

**后端**:
- Rust 2021 Edition
- Tauri 2.x
- Tokio（异步运行时）
- Axum（Web 服务器）
- Tokio-Tungstenite（WebSocket）
- Serde（序列化）
- UUID（设备 ID 生成）

**前端**:
- Vite 6.x
- TypeScript 5.6.x
- QRCode.js（二维码生成）

**平台特定依赖**:
- Windows: `winapi`, `clipboard-win`
- Linux: `x11`, `clipboard`
- macOS: `objc`, `cocoa`, `core-foundation`, `core-graphics`

## 注意事项

- 修改 `index.html` 或前端代码后，`npm run tauri:dev` 会自动重载
- Rust 代码修改需要重新编译
- 跨平台开发需要注意平台特定的虚拟键盘实现
- 版本号需要同时更新 `Cargo.toml` 和 `tauri.conf.json`
