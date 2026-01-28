# WebInput

**手机端远程输入到电脑工具** - 通过手机浏览器实现电脑端的远程文本输入。

## ✨ 功能特点

- 📱 **无需安装 App**：手机浏览器访问即可使用
- 🖥️ **实时预览**：输入内容实时同步到桌面预览
- 🔄 **双向同步**：桌面端编辑可同步回手机端
- 👥 **多设备支持**：同时连接多台手机设备
- 📋 **自动粘贴**：接收后自动上屏到光标位置（默认开启）
- ↵ **追加回车**：粘贴后自动发送（可选功能）
- 🌐 **多网卡支持**：支持 0.0.0.0 监听所有网卡
- 📦 **单文件发布**：无需额外依赖，下载即用
- 🔄 **自动重连**：断线后自动恢复连接

## 🎯 适用场景

### 语音输入（推荐）
1. 在手机上安装**豆包输入法**
2. 使用语音输入完成内容
3. 内容自动上屏到电脑光标位置

豆包输入法的中文语音识别准确率远超其他同类产品，能够：
- ✅ 准确识别中文语音
- ✅ 智能添加标点符号
- ✅ 理解上下文语义
- ✅ 支持专业术语和长文本

### 其他输入方式
- 手动输入文本
- 复制粘贴内容
- 扫码输入

## 🏗️ 系统架构

```
┌─────────────────┐     WebSocket      ┌─────────────────┐
│   桌面端应用      │ ←───────────────→ │   手机端浏览器    │
│   (Tauri/Rust)  │      Hub 模式       │   (Web/JS)      │
└─────────────────┘                    └─────────────────┘
        │                                         │
        ▼                                         ▼
┌─────────────────┐                    ┌─────────────────┐
│  虚拟键盘输入     │                    │   实时预览界面    │
│  (平台 API)     │                    │   (index.html)  │
└─────────────────┘                    └─────────────────┘
```

### 核心组件

**桌面端 (Tauri + Rust)**:
- Tauri 2.x - 桌面应用框架
- Axum - HTTP 服务器
- Tokio-Tungstenite - WebSocket
- 平台 API - 虚拟键盘输入

**手机端 (Web)**:
- 原生 JavaScript - 无需框架
- WebSocket API - 实时通信
- QRCode.js - 二维码生成

## 📁 项目结构

```
webinput/
├── frontend/                    # 前端源文件
│   ├── index.html              # 手机端界面（桌面端也使用）
│   ├── main.js                 # 前端逻辑
│   ├── vite.config.js          # Vite 配置
│   └── package.json            # 前端依赖
│
├── src-tauri/                   # Rust 后端
│   ├── src/
│   │   ├── main.rs              # Rust 应用入口
│   │   ├── lib.rs               # Tauri 库入口，注册 commands
│   │   ├── app.rs               # 应用生命周期管理
│   │   ├── commands.rs          # Tauri 命令定义
│   │   ├── server.rs            # Axum HTTP 服务器
│   │   ├── hub.rs               # WebSocket Hub，连接和消息路由
│   │   ├── types.rs             # WebSocket 消息类型定义
│   │   ├── virtual_keyboard.rs  # 虚拟键盘实现（跨平台）
│   │   └── config.rs            # 配置管理
│   ├── assets/
│   │   └── mobile.html          # 手机端独立页面（备用）
│   ├── icons/                   # 应用图标
│   ├── Cargo.toml               # Rust 依赖配置
│   ├── tauri.conf.json          # Tauri 应用配置
│   └── build.rs                # 构建脚本
│
├── .github/workflows/          # GitHub Actions
│   ├── release-windows.yml     # Windows 发布
│   └── release-multi-platform.yml  # 多平台发布
│
├── package.json                # 主项目配置
├── CLAUDE.md                   # AI 助手配置
└── README.md                   # 项目文档
```

## 🔧 开发

### 环境要求

- Rust 1.85+
- Node.js 25+
- Tauri CLI 2.x

### 安装依赖

```bash
npm install
```

### 开发模式

```bash
npm run tauri:dev
```

支持热重载，修改前端代码后自动刷新。

### 生产构建

```bash
npm run tauri:build
```

构建产物位于：
- Windows: `src-tauri/target/release/webinput.exe`
- macOS: `src-tauri/target/release/webinput`
- Linux: `src-tauri/target/release/webinput`

## 📦 发布

使用 GitHub Actions 自动发布多平台版本：

1. 推送代码到 GitHub
2. 进入 Actions → 选择工作流
3. 点击 "Run workflow"
4. 输入版本号（如 `v1.0.0`）
5. 构建完成后自动创建 Release

## 💡 使用说明

### 基本使用

1. 运行 `webinput.exe`
2. 选择 IP 地址和端口
3. 点击"启动服务"
4. 手机扫描二维码或输入地址
5. 在手机上输入内容
6. 点击"发送"或开启"自动粘贴"

### 功能开关

- **自动粘贴**：接收文本后自动上屏到光标位置
- **追加回车**：上屏后自动发送（模拟回车键）

### 网卡说明

- 选择具体 IP：生成对应二维码，局域网访问
- 选择 0.0.0.0：不生成二维码，需手动输入地址

## 🔐 技术栈

**后端**:
- Rust 2021 Edition
- Tauri 2.x
- Tokio（异步运行时）
- Axum（Web 服务器）
- Tokio-Tungstenite（WebSocket）
- Serde（序列化）

**前端**:
- Vite 6.x（构建工具）
- TypeScript 5.6.x
- QRCode.js（二维码生成）

**平台特定依赖**:
- Windows: `winapi`, `clipboard-win`
- Linux: `x11`, `x11/xtest`
- macOS: `objc`, `cocoa`, `core-foundation`, `core-graphics`

## 📜 WebSocket 消息协议

| 类型 | 方向 | 说明 |
|------|------|------|
| `connect` | 客户端 → Hub | 连接声明，标识客户端类型 |
| `preview` | 手机 → 桌面 | 预览输入内容 |
| `sync` | 桌面 → 手机 | 同步编辑内容到手机 |
| `send` | 手机 → 桌面 | 确认发送，支持追加回车 |
| `clear` | 桌面 → 手机 | 清空指定设备的输入框 |
| `history` | 手机 → 桌面 | 添加到历史记录 |

## 📄 许可证

MIT License
