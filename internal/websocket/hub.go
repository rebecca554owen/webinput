package websocket

import (
	"encoding/json"
	"log"
	"sync"
	"time"

	"github.com/rebecca554owen/webinput/internal/virtualkeyboard"
)

// Hub WebSocket 连接中心管理器
type Hub struct {
	desktop      *ClientConn
	mobiles      map[*ClientConn]bool
	mobilesMutex sync.RWMutex
	desktopMutex sync.RWMutex
	register     chan *ClientConn
	unregister   chan *ClientConn
	message      chan WSMessage
	// 设备会话管理
	deviceSessions map[string]*DeviceSession // device_id -> DeviceSession
	sessionMutex   sync.RWMutex
}

// DeviceSession 设备会话信息
type DeviceSession struct {
	DeviceID    string
	DeviceName  string
	LastContent string
	LastUpdate  time.Time
}

// NewHub 创建新的 Hub
func NewHub() *Hub {
	return &Hub{
		mobiles:         make(map[*ClientConn]bool),
		register:        make(chan *ClientConn),
		unregister:      make(chan *ClientConn),
		message:         make(chan WSMessage, 256),
		deviceSessions:  make(map[string]*DeviceSession),
	}
}

// Run 运行 Hub 的事件循环
func (h *Hub) Run() {
	defer func() {
		if r := recover(); r != nil {
			log.Printf(" Hub panic recovered: %v", r)
		}
	}()

	for {
		select {
		case client := <-h.register:
			h.handleRegister(client)

		case client := <-h.unregister:
			h.handleUnregister(client)

		case msg := <-h.message:
			h.handleMessage(msg)
		}
	}
}

// handleRegister 处理客户端注册
func (h *Hub) handleRegister(client *ClientConn) {
	if client.client.Type == ClientTypeDesktop {
		h.desktopMutex.Lock()
		// 关闭旧的桌面端连接
		if h.desktop != nil {
			log.Println("关闭旧的桌面端连接")
			h.desktop.conn.Close()
		}
		h.desktop = client
		h.desktopMutex.Unlock()
		log.Printf("桌面端已连接: %s", client.client.ID)
	} else {
		h.mobilesMutex.Lock()
		h.mobiles[client] = true
		h.mobilesMutex.Unlock()
		log.Printf("手机端已连接: %s (IP: %s, 设备: %s)", client.client.ID, client.client.IP, client.client.UserAgent)

		// 发送IP信息给手机端
		ipMsg := WSMessage{
			Type: MessageConnect,
			Data: map[string]interface{}{
				"client_ip": client.client.IP,
			},
		}
		data, _ := json.Marshal(ipMsg)
		select {
		case client.send <- data:
		default:
			log.Println("发送IP信息到手机端失败")
		}
	}
}

// handleUnregister 处理客户端注销
func (h *Hub) handleUnregister(client *ClientConn) {
	if client.client.Type == ClientTypeDesktop {
		h.desktopMutex.Lock()
		if h.desktop == client {
			h.desktop = nil
			log.Printf("桌面端已断开: %s", client.client.ID)
		}
		h.desktopMutex.Unlock()
	} else {
		h.mobilesMutex.Lock()
		delete(h.mobiles, client)
		h.mobilesMutex.Unlock()
		log.Printf("手机端已断开: %s", client.client.ID)
	}

	// 安全关闭发送通道
	select {
	case <-client.send:
		// 通道已关闭
	default:
		close(client.send)
	}
}

// handleMessage 处理消息
func (h *Hub) handleMessage(msg WSMessage) {
	defer func() {
		if r := recover(); r != nil {
			log.Printf(" handleMessage panic recovered: %v", r)
		}
	}()

	switch msg.Type {
	case MessagePreview:
		// 手机端预览 -> 桌面端显示（双向同步）
		// 保存设备会话
		if previewData, ok := msg.Data.(map[string]interface{}); ok {
			if deviceID, ok := previewData["device_id"].(string); ok {
				deviceName := ""
				if name, ok := previewData["device_name"].(string); ok {
					deviceName = name
				}
				text := ""
				if text, ok := previewData["text"].(string); ok {
					text = text
				}
				h.UpdateDeviceSession(deviceID, deviceName, text)
			}
		}

		h.desktopMutex.RLock()
		if h.desktop != nil {
			data, err := json.Marshal(msg)
			if err != nil {
				log.Printf("序列化预览消息失败: %v", err)
			} else {
				select {
				case h.desktop.send <- data:
				default:
					log.Println("桌面端发送通道已满")
				}
			}
		} else {
			log.Println("没有桌面端连接")
		}
		h.desktopMutex.RUnlock()

	case MessageSync:
		// 桌面端编辑同步 -> 广播到所有手机端
		data, err := json.Marshal(WSMessage{Type: MessageSync, Data: msg.Data})
		if err != nil {
			log.Printf("序列化同步消息失败: %v", err)
		} else {
			h.BroadcastToMobiles(data)
		}

	case MessageSend:
		// 手机端确认发送 -> 调用虚拟键盘
		if previewData, ok := msg.Data.(map[string]interface{}); ok {
			if text, ok := previewData["text"].(string); ok {
				log.Printf("发送文本到虚拟键盘: %s", text)
				if err := virtualkeyboard.PasteText(text); err != nil {
					log.Printf("虚拟键盘发送失败: %v", err)
				}
			}
		}

	case MessageClear:
		// 清空消息：只发送到桌面端，不广播到其他手机端
		data, err := json.Marshal(msg)
		if err != nil {
			log.Printf("序列化清空消息失败: %v", err)
		} else {
			// 只发送到桌面端
			h.desktopMutex.RLock()
			if h.desktop != nil {
				select {
				case h.desktop.send <- data:
				default:
					log.Println("桌面端发送通道已满")
				}
			}
			h.desktopMutex.RUnlock()
		}

	case MessageHistory:
		// 手机端手动发送成功 -> 通知桌面端添加到历史记录
		h.desktopMutex.RLock()
		if h.desktop != nil {
			data, err := json.Marshal(msg)
			if err != nil {
				log.Printf("序列化历史消息失败: %v", err)
			} else {
				select {
				case h.desktop.send <- data:
				default:
					log.Println("桌面端发送通道已满")
				}
			}
		} else {
			log.Println("没有桌面端连接")
		}
		h.desktopMutex.RUnlock()

	case MessageConnect:
		log.Printf("客户端连接声明: %+v", msg.Data)
		// 如果是手机端连接，推送会话恢复消息
		if connectData, ok := msg.Data.(map[string]interface{}); ok {
			if deviceType, ok := connectData["type"].(string); ok && deviceType == "mobile" {
				if deviceID, ok := connectData["device_id"].(string); ok {
					if session, exists := h.GetDeviceSession(deviceID); exists && session.LastContent != "" {
						// 推送恢复会话消息到桌面端
						h.desktopMutex.RLock()
						if h.desktop != nil {
							recoveryMsg := WSMessage{
								Type: MessagePreview,
								Data: map[string]interface{}{
									"text":        session.LastContent,
									"length":      len(session.LastContent),
									"device_name": session.DeviceName,
									"device_id":   session.DeviceID,
									"restore":     true,
								},
							}
							data, _ := json.Marshal(recoveryMsg)
							select {
							case h.desktop.send <- data:
								log.Printf("恢复设备会话: %s", session.DeviceName)
							default:
								log.Println("桌面端发送通道已满")
							}
						}
						h.desktopMutex.RUnlock()
					}
				}
			}
		}
	}
}

// GetOrCreateDeviceSession 获取或创建设备会话
func (h *Hub) GetOrCreateDeviceSession(deviceID, deviceName string) *DeviceSession {
	h.sessionMutex.Lock()
	defer h.sessionMutex.Unlock()

	session, exists := h.deviceSessions[deviceID]
	if !exists {
		session = &DeviceSession{
			DeviceID:   deviceID,
			DeviceName: deviceName,
			LastContent: "",
			LastUpdate:  time.Now(),
		}
		h.deviceSessions[deviceID] = session
		log.Printf("创建新设备会话: %s (%s)", deviceName, deviceID)
	}

	return session
}

// UpdateDeviceSession 更新设备会话
func (h *Hub) UpdateDeviceSession(deviceID, deviceName, content string) {
	h.sessionMutex.Lock()
	defer h.sessionMutex.Unlock()

	session, exists := h.deviceSessions[deviceID]
	if !exists {
		session = &DeviceSession{
			DeviceID:   deviceID,
			DeviceName: deviceName,
		}
		h.deviceSessions[deviceID] = session
	}

	if deviceName != "" {
		session.DeviceName = deviceName
	}
	session.LastContent = content
	session.LastUpdate = time.Now()
}

// GetDeviceSession 获取设备会话
func (h *Hub) GetDeviceSession(deviceID string) (*DeviceSession, bool) {
	h.sessionMutex.RLock()
	defer h.sessionMutex.RUnlock()

	session, exists := h.deviceSessions[deviceID]
	return session, exists
}

// BroadcastToMobiles 广播消息到所有手机端
func (h *Hub) BroadcastToMobiles(data []byte) {
	h.mobilesMutex.RLock()
	defer h.mobilesMutex.RUnlock()

	for client := range h.mobiles {
		select {
		case client.send <- data:
			// 发送成功
		default:
			// 发送通道已满或阻塞，跳过这个客户端
			log.Printf("手机端发送通道阻塞，跳过: %s", client.client.ID)
		}
	}
}
