import { invoke } from '@tauri-apps/api/core';

const sentHistory = [];
const receivedHistory = [];

class DesktopWebSocket {
    constructor() {
        this.ws = null;
        this.devices = new Map();
        this.reconnectTimer = null;
    }

    async connect(accessURL) {
        if (!isServerRunning) {
            return;
        }

        const wsProtocol = accessURL.startsWith('https') ? 'wss:' : 'ws:';
        const wsURL = `${wsProtocol}//${new URL(accessURL).host}/ws?type=desktop`;

        try {
            this.ws = new WebSocket(wsURL);

            this.ws.onopen = () => {
                if (!isServerRunning) {
                    this.ws.close();
                    return;
                }
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

            this.ws.onclose = async () => {
                this.updateConnectionStatus(false);
                const running = await invoke('is_running');
                if (running !== isServerRunning) {
                    isServerRunning = running;
                    updateButtonStates();
                }
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

                if (autoPasteEnabled) {
                    this.pasteToDevice(msg.data.text);
                }
            }
            break;
        case 'clear':
            if (msg.data && msg.data.device_id) {
                this.clearDevicePreview(msg.data.device_id);
            }
            break;
        }
    }

    async pasteToDevice(text) {
        if (!text) return;
        this.send({
            type: 'send',
            data: {
                text: text,
                append_enter: autoEnterEnabled
            }
        });
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

            if (autoPasteEnabled) {
                this.send({
                    type: 'send',
                    data: {
                        text: text,
                        append_enter: autoEnterEnabled
                    }
                });
            }
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
        if (!isServerRunning) {
            return;
        }
        const statusDot = document.getElementById('statusDot');
        const statusText = document.getElementById('statusText');

        if (connected) {
            statusDot.classList.add('running');
            statusDot.style.background = '';
            statusText.textContent = '运行中';
        } else {
            statusDot.classList.add('running');
            statusDot.style.background = '#ff9500';
            statusText.textContent = '实时预览失败';
        }
    }

    scheduleReconnect() {
    // 不再使用重连逻辑，用户可以手动重新启动服务
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
let autoPasteEnabled = true;
let autoEnterEnabled = false;

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
        const ips = await invoke('get_ips');
        const mainIP = await invoke('get_main_ip');

        selectedIP = mainIP;
        document.getElementById('ipDisplay').textContent = mainIP;

        populateIPSelect(ips);

        const portInput = document.getElementById('portInput');
        const port = await invoke('get_port');
        portInput.value = port;

        isServerRunning = await invoke('is_running');

        const autoPaste = document.getElementById('autoPaste');
        const autoEnter = document.getElementById('autoEnter');

        const savedAutoPaste = await invoke('get_auto_paste').catch(() => true);
        const savedAutoEnter = await invoke('get_auto_enter').catch(() => false);

        autoPaste.checked = savedAutoPaste;
        autoEnter.checked = savedAutoEnter;
        autoPasteEnabled = savedAutoPaste;
        autoEnterEnabled = savedAutoEnter;

        autoPaste.addEventListener('change', async (e) => {
            autoPasteEnabled = e.target.checked;
            await invoke('set_auto_paste', { enabled: autoPasteEnabled }).catch(() => {});
        });

        autoEnter.addEventListener('change', async (e) => {
            autoEnterEnabled = e.target.checked;
            await invoke('set_auto_enter', { enabled: autoEnterEnabled }).catch(() => {});
        });

        updateButtonStates();

        if (isServerRunning) {
            // 验证服务器是否真的在运行
            try {
                const accessURL = await invoke('get_access_url');
                // 尝试连接 WebSocket 来验证服务器状态
                // 如果连接失败，说明服务器实际没运行
                const testWS = new WebSocket(accessURL.replace('http://', 'ws://').replace('https://', 'wss://') + '/ws');

                const connectionTest = new Promise((resolve) => {
                    const timeout = setTimeout(() => resolve(false), 1000);
                    testWS.onopen = () => {
                        clearTimeout(timeout);
                        testWS.close();
                        resolve(true);
                    };
                    testWS.onerror = () => {
                        clearTimeout(timeout);
                        resolve(false);
                    };
                    testWS.onclose = () => {
                        clearTimeout(timeout);
                        resolve(false);
                    };
                });

                const serverAlive = await connectionTest;

                if (serverAlive) {
                    showQRCode(accessURL);
                    desktopWS.connect(accessURL);
                } else {
                    // 服务器实际没运行，重置状态
                    isServerRunning = false;
                    await invoke('stop_server').catch(() => {});
                    updateButtonStates();
                }
            } catch (e) {
                // 出错时也重置状态
                isServerRunning = false;
                await invoke('stop_server').catch(() => {});
                updateButtonStates();
            }
        }
    } catch {}
}

function populateIPSelect(ips) {
    const select = document.getElementById('ipSelect');
    if (!select) return;

    select.innerHTML = '';

    if (!ips || ips.length === 0) return;

    ips.forEach(ip => {
        const option = document.createElement('option');
        option.value = ip;
        option.textContent = ip === '0.0.0.0' ? '0.0.0.0 (所有网卡)' : ip;
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
        statusDot.style.background = '';
        statusText.textContent = '运行中';
        startBtn.disabled = true;
        stopBtn.disabled = false;
        startBtn.style.opacity = '0.5';
        stopBtn.style.opacity = '1';
        stopBtn.classList.remove('btn-secondary');
        stopBtn.classList.add('btn-danger');
    } else {
        statusDot.classList.remove('running');
        statusDot.style.background = '';
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

    document.querySelector('.qrcode-container').classList.add('visible');

    accessUrlDiv.textContent = accessURL;
    accessUrlDiv.style.display = 'block';

    accessUrlDiv.onclick = async () => {
        await navigator.clipboard.writeText(accessURL);
        const originalText = accessUrlDiv.textContent;
        accessUrlDiv.textContent = '✓ 已复制';
        accessUrlDiv.style.color = '#34C759';
        setTimeout(() => {
            accessUrlDiv.textContent = accessURL;
            accessUrlDiv.style.color = '#3b82f6';
        }, 1000);
    };

    qrContainer.innerHTML = '';

    const url = new URL(accessURL);
    const hostname = url.hostname;

    if (hostname !== '0.0.0.0') {
        new QRCode(qrContainer, {
            text: accessURL,
            width: 200,
            height: 200,
            colorDark: '#000000',
            colorLight: '#ffffff',
            correctLevel: QRCode.CorrectLevel.H
        });
    } else {
        qrContainer.innerHTML = '<p style="color: #6b7280; font-size: 13px; margin-top: 60px;">0.0.0.0 模式不生成二维码<br>请手动输入地址</p>';
    }
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
        await invoke('set_selected_ip', { ip: selectedIP });
    }
}, true);

document.getElementById('startBtn').addEventListener('click', async () => {
    const startBtn = document.getElementById('startBtn');
    startBtn.disabled = true;

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

    await invoke('stop_server');

    isServerRunning = await invoke('is_running');

    updateButtonStates();

    document.querySelector('.qrcode-container').classList.remove('visible');
    desktopWS.disconnect();
});
