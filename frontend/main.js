const sentHistory = [];
const receivedHistory = [];

async function getInvoke() {
    return window.__TAURI__.core.invoke;
}

class DesktopWebSocket {
    constructor() {
        this.ws = null;
        this.devices = new Map();
        this.reconnectTimer = null;
    }

    async connect(accessURL) {
        const wsProtocol = accessURL.startsWith('https') ? 'wss:' : 'ws:';
        const wsURL = `${wsProtocol}//${new URL(accessURL).host}/ws?type=desktop`;

        try {
            this.ws = new WebSocket(wsURL);

            this.ws.onopen = () => {
                this.updateConnectionStatus(true);
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
                this.updateConnectionStatus(false);
                this.scheduleReconnect(accessURL);
            };

            this.ws.onerror = () => {};
        } catch {}
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
            if (msg.data && msg.data.text) {
                const deviceName = msg.data.device_name || '未知设备';
                const clientIP = msg.data.client_ip || '';
                const displayName = clientIP ? `${deviceName} (${clientIP})` : deviceName;
                addReceivedHistory(msg.data.text, displayName);
            }
            break;
        case 'clear':
            if (msg.data && msg.data.device_id) {
                this.clearDevicePreview(msg.data.device_id);
            }
            break;
        }
    }

    showPreview(data) {
        const deviceId = data.device_id || 'unknown';
        const deviceName = data.device_name || '未知设备';
        const clientIP = data.client_ip || '';
        const displayName = clientIP ? `${deviceName} (${clientIP})` : deviceName;

        if (!this.devices.has(deviceId)) {
            this.createPreviewBox(deviceId, displayName, clientIP);
        }

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

        this.devices.set(deviceId, {
            textarea,
            lengthSpan,
            isEditing: false,
            deviceDisplayName: displayName,
            clientIP: clientIP
        });
    }

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

    async copyFromDevice(deviceId) {
        const device = this.devices.get(deviceId);
        if (!device) return;

        const text = device.textarea.value;
        if (!text) return;

        try {
            await navigator.clipboard.writeText(text);

            const section = document.getElementById(`preview-${deviceId}`);
            const copyBtn = section.querySelector('.btn-primary');
            const originalText = copyBtn.textContent;
            copyBtn.textContent = '已复制!';
            setTimeout(() => {
                copyBtn.textContent = originalText;
            }, 1000);

            addReceivedHistory(text, device.deviceDisplayName || '未知设备');
        } catch {}
    }

    clearDevice(deviceId) {
        const device = this.devices.get(deviceId);
        if (!device) return;

        device.textarea.value = '';
        device.lengthSpan.textContent = '0 字符';

        this.send({
            type: 'clear',
            data: { device_id: deviceId }
        });
    }

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

const desktopWS = new DesktopWebSocket();

let selectedIP = '';
let isServerRunning = false;

window.addEventListener('DOMContentLoaded', () => {
    const versionEl = document.getElementById('appVersion');
    if (versionEl && typeof __APP_VERSION__ !== 'undefined') {
        versionEl.textContent = __APP_VERSION__;
    }

    const clearBtn = document.getElementById('clearHistoryBtn');
    if (clearBtn) {
        clearBtn.addEventListener('click', () => {
            receivedHistory.length = 0;
            updateHistoryDisplay();
        });
    }

    initApp();
});

async function initApp() {
    try {
        const invoke = await getInvoke();

        const ips = await invoke('get_ips');
        const mainIP = await invoke('get_main_ip');

        selectedIP = mainIP;
        document.getElementById('ipDisplay').textContent = mainIP;

        populateIPSelect(ips);

        const portInput = document.getElementById('portInput');
        const port = await invoke('get_port');
        portInput.value = port;

        isServerRunning = await invoke('is_running');

        updateButtonStates();

        if (isServerRunning) {
            const accessURL = await invoke('get_access_url');
            showQRCode(accessURL);
            desktopWS.connect(accessURL);
        }
    } catch {}
}

function populateIPSelect(ips) {
    const select = document.getElementById('ipSelect');
    if (!select) return;

    select.innerHTML = '';

    if (!ips || ips.length === 0) return;

    ips.forEach(ip => {
        if (ip === '0.0.0.0') return;
        const option = document.createElement('option');
        option.value = ip;
        option.textContent = ip;
        if (ip === selectedIP) {
            option.selected = true;
        }
        select.appendChild(option);
    });
}

function updateButtonStates() {
    const startBtn = document.getElementById('startBtn');
    const stopBtn = document.getElementById('stopBtn');
    const statusDot = document.getElementById('statusDot');
    const statusText = document.getElementById('statusText');

    if (isServerRunning) {
        statusDot.classList.add('running');
        statusText.textContent = '运行中';
        startBtn.disabled = true;
        stopBtn.disabled = false;
        startBtn.style.opacity = '0.5';
        stopBtn.style.opacity = '1';
        stopBtn.classList.remove('btn-secondary');
        stopBtn.classList.add('btn-danger');
    } else {
        statusDot.classList.remove('running');
        statusText.textContent = '未启动';
        startBtn.disabled = false;
        stopBtn.disabled = true;
        startBtn.style.opacity = '1';
        stopBtn.style.opacity = '0.5';
        stopBtn.classList.remove('btn-danger');
        stopBtn.classList.add('btn-secondary');
    }
}

function showQRCode(accessURL) {
    const qrContainer = document.getElementById('qrcode');
    const accessUrlDiv = document.getElementById('accessUrl');
    const copyLink = document.getElementById('copyLink');

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

    accessUrlDiv.onclick = async () => {
        await navigator.clipboard.writeText(accessURL);
        const originalText = accessUrlDiv.textContent;
        accessUrlDiv.textContent = '✓ 已复制';
        setTimeout(() => {
            accessUrlDiv.textContent = accessURL;
        }, 1000);
    };
}

function addReceivedHistory(text, deviceName) {
    const item = { text, deviceName, timestamp: Date.now() };
    receivedHistory.unshift(item);
    if (receivedHistory.length > 20) receivedHistory.pop();
    updateHistoryDisplay();
}

function updateHistoryDisplay() {
    const historyList = document.getElementById('historyList');
    historyList.innerHTML = '';

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

    if (receivedHistory.length === 0) {
        const li = document.createElement('li');
        li.className = 'history-item';
        li.style.justifyContent = 'center';
        li.style.color = '#9ca3af';
        li.textContent = '暂无接收记录';
        historyList.appendChild(li);
    }
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

document.addEventListener('change', async (e) => {
    if (e.target && e.target.id === 'ipSelect') {
        selectedIP = e.target.value;
        document.getElementById('ipDisplay').textContent = selectedIP;
        const invoke = await getInvoke();
        await invoke('set_selected_ip', { ip: selectedIP });
    }
}, true);

document.getElementById('startBtn').addEventListener('click', async () => {
    const startBtn = document.getElementById('startBtn');
    startBtn.disabled = true;

    const invoke = await getInvoke();
    const portInput = document.getElementById('portInput');
    const port = portInput.value;
    await invoke('set_port', { port });

    await invoke('start_server');

    isServerRunning = await invoke('is_running');

    updateButtonStates();

    const accessURL = await invoke('get_access_url');
    showQRCode(accessURL);
    desktopWS.connect(accessURL);
});

document.getElementById('stopBtn').addEventListener('click', async () => {
    const stopBtn = document.getElementById('stopBtn');
    stopBtn.disabled = true;

    const invoke = await getInvoke();
    await invoke('stop_server');

    isServerRunning = await invoke('is_running');

    updateButtonStates();

    document.querySelector('.qrcode-container').classList.remove('visible');
    document.getElementById('copyLink').style.display = 'none';
    desktopWS.disconnect();
});

document.getElementById('copyLink').addEventListener('click', async () => {
    const invoke = await getInvoke();
    const accessURL = await invoke('get_access_url');
    await navigator.clipboard.writeText(accessURL);
    const copyLink = document.getElementById('copyLink');
    copyLink.textContent = '已复制!';
    setTimeout(() => {
        copyLink.textContent = '复制链接';
    }, 1500);
});
