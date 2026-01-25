package websocket

import (
	"encoding/json"
	"log"
	"time"

	"github.com/gorilla/websocket"
)

const (
	// WriteWait 等待写入超时时间
	WriteWait = 10 * time.Second
	// PongWait 等待pong响应超时时间
	PongWait = 60 * time.Second
	// PingPeriod 发送ping的时间间隔
	PingPeriod = (PongWait * 9) / 10 // 54秒
)

// ClientConn WebSocket 客户端连接
type ClientConn struct {
	hub    *Hub
	conn   *websocket.Conn
	send   chan []byte
	client Client
}

// Client 客户端信息
type Client struct {
	Type        ClientType
	ID          string
	UserAgent   string
	IP          string
	ConnectedAt time.Time
}

// NewClientConn 创建新的客户端连接
func NewClientConn(hub *Hub, conn *websocket.Conn, client Client) *ClientConn {
	return &ClientConn{
		hub:    hub,
		conn:   conn,
		send:   make(chan []byte, 256),
		client: client,
	}
}

// ReadPump 读取泵，从 WebSocket 连接读取消息
func (c *ClientConn) ReadPump() {
	defer func() {
		c.hub.unregister <- c
		c.conn.Close()
		// 不在这里关闭 send 通道，由 handleUnregister 统一关闭
	}()

	c.conn.SetReadDeadline(time.Now().Add(PongWait))
	c.conn.SetPongHandler(func(string) error {
		c.conn.SetReadDeadline(time.Now().Add(PongWait))
		return nil
	})

	for {
		_, message, err := c.conn.ReadMessage()
		if err != nil {
			if websocket.IsUnexpectedCloseError(err, websocket.CloseGoingAway, websocket.CloseAbnormalClosure) {
				log.Printf("WebSocket 读取错误: %v", err)
			}
			break
		}

		var msg WSMessage
		if err := json.Unmarshal(message, &msg); err != nil {
			log.Printf("解析消息失败: %v", err)
			continue
		}
		msg.ClientID = c.client.ID
		c.hub.message <- msg
	}
}

// WritePump 写入泵，向 WebSocket 连接写入消息
func (c *ClientConn) WritePump() {
	ticker := time.NewTicker(PingPeriod)
	defer func() {
		ticker.Stop()
		c.conn.Close()
	}()

	for {
		select {
		case message, ok := <-c.send:
			c.conn.SetWriteDeadline(time.Now().Add(WriteWait))
			if !ok {
				// Hub 关闭了通道
				c.conn.WriteMessage(websocket.CloseMessage, []byte{})
				return
			}

			if err := c.conn.WriteMessage(websocket.TextMessage, message); err != nil {
				log.Printf("WebSocket 写入错误: %v", err)
				return
			}

		case <-ticker.C:
			c.conn.SetWriteDeadline(time.Now().Add(WriteWait))
			if err := c.conn.WriteMessage(websocket.PingMessage, nil); err != nil {
				return
			}
		}
	}
}
