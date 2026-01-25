// 最近发送的记录（最多20）条
const sentHistory = [];
// 接收记录（手机端预览后确认发送）
const receivedHistory = [];

// ========== WebSocket 客户端 ==========
class DesktopWebSocket {
    constructor() {
        this.ws = null;
        this.devices = new Map(); // 设备 ID -> 设备信息
        this.reconnectTimer = null;
    }

    async connect(accessURL) {
        const wsProtocol = accessURL.startsWith('https') ? 'wss:' : 'ws:';
        const wsURL = `${wsProtocol}//${new URL(accessURL).host}/ws?type=desktop`;

        try {
            this.ws = new WebSocket(wsURL);

            this.ws.onopen = () => {
                console.log('桌面端 WebSocket 已连接');
                this.updateConnectionStatus(true);

                // 声明为桌面端
                this.send({
                    type: 'connect',
                    data: { type: 'desktop' }
                });
            };

            this.ws.onmessage = (event) => {
                const msg = JSON.parse(event.data);
                this.handleMessage(msg);
            };

            this.ws.onclose = () => {
                console.log('桌面端 WebSocket 已断开');
                this.updateConnectionStatus(false);
                this.scheduleReconnect(accessURL);
            };

            this.ws.onerror = (error) => {
                console.error('桌面端 WebSocket 错误:', error);
            };
        } catch (error) {
            console.error('WebSocket 连接失败:', error);
        }
    }

    send(data) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify({
                ...data,
                timestamp: Date.now()
            }));
        }
    }

    handleMessage(msg) {
        switch (msg.type) {
        case 'preview':
            this.showPreview(msg.data);
            break;
        case 'history':
            // 手机端手动发送成功，添加到桌面历史记录
            if (msg.data && msg.data.text) {
                const deviceName = msg.data.device_name || '未知设备';
                const clientIP = msg.data.client_ip || '';
                const displayName = clientIP ? `${deviceName} (${clientIP})` : deviceName;
                addReceivedHistory(msg.data.text, displayName);
            }
            break;
        case 'clear':
            // 清空对应设备的预览框
            if (msg.data && msg.data.device_id) {
                this.clearDevicePreview(msg.data.device_id);
            }
            break;
        case 'connect':
            console.log('新客户端连接:', msg.data);
            break;
        }
    }

    showPreview(data) {
        const deviceId = data.device_id || 'unknown';
        const deviceName = data.device_name || '未知设备';
        const clientIP = data.client_ip || '';

        // 构建设备显示名称（包含IP）
        const displayName = clientIP ? `${deviceName} (${clientIP})` : deviceName;

        // 如果设备不存在，创建新的预览框
        if (!this.devices.has(deviceId)) {
            this.createPreviewBox(deviceId, displayName, clientIP);
        }

        // 更新预览框内容
        const device = this.devices.get(deviceId);
        if (!device.isEditing) {
            device.textarea.value = data.text || '';
        }
        device.lengthSpan.textContent = `${data.length} 字符`;
    }

    createPreviewBox(deviceId, displayName, clientIP) {
        const container = document.getElementById('previewContainer');

        const section = document.createElement('div');
        section.className = 'preview-section';
        section.id = `preview-${deviceId}`;

        const header = document.createElement('h3');
        header.textContent = displayName;

        const textarea = document.createElement('textarea');
        textarea.className = 'preview-content';
        textarea.placeholder = '等待输入...';

        const meta = document.createElement('div');
        meta.className = 'preview-meta';

        const lengthSpan = document.createElement('span');
        lengthSpan.id = `length-${deviceId}`;
        lengthSpan.textContent = '0 字符';

        const actions = document.createElement('div');
        actions.className = 'preview-actions';

        const copyBtn = document.createElement('button');
        copyBtn.className = 'btn-primary';
        copyBtn.style.padding = '8px 16px';
        copyBtn.textContent = '复制';
        copyBtn.onclick = () => this.copyFromDevice(deviceId);

        const clearBtn = document.createElement('button');
        clearBtn.className = 'btn-secondary';
        clearBtn.style.padding = '8px 16px';
        clearBtn.textContent = '清空';
        clearBtn.onclick = () => this.clearDevice(deviceId);

        actions.appendChild(copyBtn);
        actions.appendChild(clearBtn);

        meta.appendChild(lengthSpan);
        meta.appendChild(actions);

        section.appendChild(header);
        section.appendChild(textarea);
        section.appendChild(meta);

        container.appendChild(section);

        // 添加输入监听
        let syncTimer = null;
        textarea.addEventListener('input', () => {
            const device = this.devices.get(deviceId);
            device.isEditing = true;

            if (syncTimer) clearTimeout(syncTimer);
            syncTimer = setTimeout(() => {
                this.syncToDevice(deviceId, textarea.value);
            }, 100);
        });

        textarea.addEventListener('focus', () => {
            const device = this.devices.get(deviceId);
            device.isEditing = true;
        });

        textarea.addEventListener('blur', () => {
            setTimeout(() => {
                const device = this.devices.get(deviceId);
                if (device) device.isEditing = false;
            }, 200);
        });

        // 保存到 devices map
        this.devices.set(deviceId, {
            textarea,
            lengthSpan,
            isEditing: false,
            deviceDisplayName: displayName,
            clientIP: clientIP
        });
    }

    // 编辑时同步到手机端
    syncToDevice(deviceId, text) {
        this.send({
            type: 'sync',
            data: { text: text, device_id: deviceId }
        });
        const device = this.devices.get(deviceId);
        if (device) {
            device.lengthSpan.textContent = `${text.length} 字符`;
        }
    }

    // 从设备复制到剪贴板
    async copyFromDevice(deviceId) {
        const device = this.devices.get(deviceId);
        if (!device) return;

        const text = device.textarea.value;
        if (!text) return;

        try {
            await navigator.clipboard.writeText(text);

            // 显示复制成功提示
            const section = document.getElementById(`preview-${deviceId}`);
            const copyBtn = section.querySelector('.btn-primary');
            const originalText = copyBtn.textContent;
            copyBtn.textContent = '已复制!';
            setTimeout(() => {
                copyBtn.textContent = originalText;
            }, 1000);

            // 添加到历史记录
            addReceivedHistory(text, device.deviceDisplayName || '未知设备');
        } catch (err) {
            console.error('复制失败:', err);
        }
    }

    // 清空设备预览
    clearDevice(deviceId) {
        const device = this.devices.get(deviceId);
        if (!device) return;

        device.textarea.value = '';
        device.lengthSpan.textContent = '0 字符';

        // 通知手机端清空
        this.send({
            type: 'clear',
            data: { device_id: deviceId }
        });
    }

    // 清空设备预览框（不发送消息，避免循环）
    clearDevicePreview(deviceId) {
        const device = this.devices.get(deviceId);
        if (!device) return;

        device.textarea.value = '';
        device.lengthSpan.textContent = '0 字符';
    }

    updateConnectionStatus(connected) {
        const statusDot = document.getElementById('statusDot');
        const statusText = document.getElementById('statusText');

        if (connected) {
            statusDot.style.background = '#34C759';
            statusText.textContent = '实时预览已连接';
        } else {
            statusDot.style.background = '#ff9500';
            statusText.textContent = '实时预览连接中...';
        }
    }

    scheduleReconnect(accessURL) {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
        }

        this.reconnectTimer = setTimeout(() => {
            this.connect(accessURL);
        }, 3000);
    }

    disconnect() {
        if (this.reconnectTimer) {
            clearTimeout(this.reconnectTimer);
        }
        if (this.ws) {
            this.ws.close();
        }
    }
}

// 创建全局实例
const desktopWS = new DesktopWebSocket();

// 当前选中的 IP
let selectedIP = '';
// 服务器实际运行状态
let isServerRunning = false;

// 初始化应用
window.addEventListener('DOMContentLoaded', () => {
    // 设置版本号显示
    const versionEl = document.getElementById('appVersion');
    if (versionEl && typeof __APP_VERSION__ !== 'undefined') {
        versionEl.textContent = __APP_VERSION__;
    }

    // 绑定历史记录清空按钮
    document.getElementById('clearHistoryBtn').addEventListener('click', () => {
        receivedHistory.length = 0;
        updateHistoryDisplay();
    });

    // 等待一小段时间确保 Wails 绑定加载完成
    setTimeout(initApp, 100);
});

// 初始化应用
async function initApp() {
    try {
        // 检查 Wails 绑定是否可用
        if (typeof window.go === 'undefined' || typeof window.go.build === 'undefined' || typeof window.go.build.App === 'undefined') {
            console.error('Wails 绑定未加载，等待重试...');
            setTimeout(initApp, 500);
            return;
        }

        console.log('Wails 绑定已加载，开始初始化');

        // 获取所有可用的 IP 地址
        const ips = await window.go.build.App.GetIPs();
        console.log('获取到的 IP 地址:', ips);

        // 获取主要 IP
        const mainIP = await window.go.build.App.GetMainIP();
        console.log('主要 IP:', mainIP);

        selectedIP = mainIP;
        document.getElementById('ipDisplay').textContent = mainIP;

        // 填充 IP 选择器（在设置 selectedIP 之后）
        populateIPSelect(ips);

        // 获取端口
        const portInput = document.getElementById('portInput');
        const port = await window.go.build.App.GetPort();
        portInput.value = port;

        // 检查服务器是否运行
        isServerRunning = await window.go.build.App.IsRunning();
        console.log('服务器运行状态:', isServerRunning);

        updateButtonStates();

        // 如果正在运行，显示二维码并连接 WebSocket
        if (isServerRunning) {
            const accessURL = await window.go.build.App.GetAccessURL();
            showQRCode(accessURL);
            desktopWS.connect(accessURL);
        }
    } catch (error) {
        console.error('初始化失败:', error);
        // 重试
        setTimeout(initApp, 1000);
    }
}

// 填充 IP 选择器
function populateIPSelect(ips) {
    const select = document.getElementById('ipSelect');
    if (!select) {
        console.error('找不到 ipSelect 元素');
        return;
    }

    select.innerHTML = '';

    if (!ips || ips.length === 0) {
        console.error('IP 地址列表为空');
        return;
    }

    ips.forEach(ip => {
        if (ip === '0.0.0.0') return; // 跳过 0.0.0.0
        const option = document.createElement('option');
        option.value = ip;
        option.textContent = ip;
        if (ip === selectedIP) {
            option.selected = true;
        }
        select.appendChild(option);
    });

    console.log('IP 选择器已填充，选项数:', select.options.length);
}

// 更新按钮状态
function updateButtonStates() {
    const startBtn = document.getElementById('startBtn');
    const stopBtn = document.getElementById('stopBtn');
    const statusDot = document.getElementById('statusDot');
    const statusText = document.getElementById('statusText');

    if (isServerRunning) {
        // 服务器运行中
        statusDot.classList.add('running');
        statusText.textContent = '运行中';
        startBtn.disabled = true;
        stopBtn.disabled = false;
        startBtn.style.opacity = '0.5';
        stopBtn.style.opacity = '1';
        // 停止按钮使用危险红色
        stopBtn.classList.remove('btn-secondary');
        stopBtn.classList.add('btn-danger');
    } else {
        // 服务器未启动
        statusDot.classList.remove('running');
        statusText.textContent = '未启动';
        startBtn.disabled = false;
        stopBtn.disabled = true;
        startBtn.style.opacity = '1';
        stopBtn.style.opacity = '0.5';
        // 停止按钮使用次要灰色
        stopBtn.classList.remove('btn-danger');
        stopBtn.classList.add('btn-secondary');
    }
}

// 显示二维码
function showQRCode(accessURL) {
    const qrContainer = document.getElementById('qrcode');
    const accessUrlDiv = document.getElementById('accessUrl');
    const copyLink = document.getElementById('copyLink');

    // 使用 classList.add 而非修改 display
    document.querySelector('.qrcode-container').classList.add('visible');

    qrContainer.innerHTML = '';
    new QRCode(qrContainer, {
        text: accessURL,
        width: 200,
        height: 200,
        colorDark: '#000000',
        colorLight: '#ffffff',
        correctLevel: QRCode.CorrectLevel.H
    });

    accessUrlDiv.textContent = accessURL;
    accessUrlDiv.style.display = 'block';
    copyLink.style.display = 'block';

    // 地址点击复制
    accessUrlDiv.onclick = async () => {
        await navigator.clipboard.writeText(accessURL);
        const originalText = accessUrlDiv.textContent;
        accessUrlDiv.textContent = '✓ 已复制';
        setTimeout(() => {
            accessUrlDiv.textContent = accessURL;
        }, 1000);
    };
}

// 添加接收记录
function addReceivedHistory(text, deviceName) {
    const item = { text, deviceName, timestamp: Date.now() };
    receivedHistory.unshift(item);
    if (receivedHistory.length > 20) receivedHistory.pop();
    updateHistoryDisplay();
}

// 更新历史记录显示
function updateHistoryDisplay() {
    const historyList = document.getElementById('historyList');
    historyList.innerHTML = '';

    // 显示接收记录（按时间倒序）
    receivedHistory.forEach(item => {
        const li = document.createElement('li');
        li.className = 'history-item';
        li.style.cursor = 'pointer';
        li.title = '点击复制';

        const escapedText = escapeHtml(item.text);
        li.innerHTML = `
            <span class="text">${escapedText}</span>
            <span class="device">📱 ${item.deviceName}</span>
        `;

        // 点击复制
        li.onclick = () => {
            navigator.clipboard.writeText(item.text);
            const originalText = li.innerHTML;
            li.style.background = '#dcfce7';
            setTimeout(() => {
                li.style.background = '#f9fafb';
            }, 500);
        };

        historyList.appendChild(li);
    });

    // 如果没有记录
    if (receivedHistory.length === 0) {
        const li = document.createElement('li');
        li.className = 'history-item';
        li.style.justifyContent = 'center';
        li.style.color = '#9ca3af';
        li.textContent = '暂无接收记录';
        historyList.appendChild(li);
    }
}

// HTML 转义函数
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// IP 选择变化
document.addEventListener('change', (e) => {
    if (e.target && e.target.id === 'ipSelect') {
        selectedIP = e.target.value;
        document.getElementById('ipDisplay').textContent = selectedIP;
        window.go.build.App.SetSelectedIP(selectedIP).catch(err => {
            console.error('设置 IP 失败:', err);
        });
    }
}, true);

// 启动服务
document.getElementById('startBtn').addEventListener('click', async () => {
    console.log('点击启动服务按钮');

    // 先禁用按钮防止重复点击
    const startBtn = document.getElementById('startBtn');
    startBtn.disabled = true;

    const portInput = document.getElementById('portInput');
    const port = portInput.value;
    await window.go.build.App.SetPort(port);

    // 启动服务器
    await window.go.build.App.StartServer();

    // 等待一会让服务器启动完成，然后刷新状态
    setTimeout(async () => {
        isServerRunning = await window.go.build.App.IsRunning();
        console.log('启动后服务器状态:', isServerRunning);

        updateButtonStates();

        const accessURL = await window.go.build.App.GetAccessURL();
        showQRCode(accessURL);
        desktopWS.connect(accessURL);
    }, 200);
});

// 停止服务
document.getElementById('stopBtn').addEventListener('click', async () => {
    console.log('点击停止服务按钮');

    // 先禁用按钮防止重复点击
    const stopBtn = document.getElementById('stopBtn');
    stopBtn.disabled = true;

    await window.go.build.App.StopServer();

    // 等待一会让服务器停止完成，然后刷新状态
    setTimeout(async () => {
        isServerRunning = await window.go.build.App.IsRunning();
        console.log('停止后服务器状态:', isServerRunning);

        updateButtonStates();

        // 隐藏二维码容器
        document.querySelector('.qrcode-container').classList.remove('visible');
        document.getElementById('copyLink').style.display = 'none';
        desktopWS.disconnect();
    }, 200);
});

// 复制链接
document.getElementById('copyLink').addEventListener('click', async () => {
    const accessURL = await window.go.build.App.GetAccessURL();
    await navigator.clipboard.writeText(accessURL);
    const copyLink = document.getElementById('copyLink');
    copyLink.textContent = '已复制!';
    setTimeout(() => {
        copyLink.textContent = '复制链接';
    }, 1500);
});

// 监听新文本到达（从 server 接收）
function onNewTextReceived(text) {
    addToHistory(text);
}
