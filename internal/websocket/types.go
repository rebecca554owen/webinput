package websocket

// MessageType WebSocket 消息类型
type MessageType string

const (
	MessagePreview MessageType = "preview" // 手机↔桌面：预览内容（双向同步）
	MessageSync    MessageType = "sync"    // 桌面→手机：同步编辑内容
	MessageSend    MessageType = "send"    // 手机→桌面：确认发送（直接上屏）
	MessageHistory MessageType = "history" // 手机→桌面：添加到历史记录
	MessageClear   MessageType = "clear"   // 桌面→手机：清空预览
	MessageConnect MessageType = "connect" // 客户端→服务器：连接声明
	MessagePing    MessageType = "ping"    // 心跳
	MessagePong    MessageType = "pong"    // 心跳响应
)

// WSMessage WebSocket 消息结构
type WSMessage struct {
	Type      MessageType   `json:"type"`
	Data      interface{}   `json:"data"`
	Timestamp int64         `json:"timestamp,omitempty"`
	ClientID  string        `json:"client_id,omitempty"`
}

// ClientType 客户端类型
type ClientType string

const (
	ClientTypeDesktop ClientType = "desktop" // 桌面端（Wails应用）
	ClientTypeMobile  ClientType = "mobile"  // 手机端（浏览器）
)

// PreviewData 预览数据
type PreviewData struct {
	Text       string `json:"text"`
	Length     int    `json:"length"`
	DeviceName string `json:"device_name,omitempty"`
	DeviceID   string `json:"device_id,omitempty"`
}
