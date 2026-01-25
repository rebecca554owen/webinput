package websocket

import (
	"log"
	"net/http"
	"strings"
	"time"

	"github.com/google/uuid"
	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		return true // 允许所有来源（跨域）
	},
}

// getClientIP 获取客户端真实IP
func getClientIP(r *http.Request) string {
	// 检查 X-Forwarded-For 头（代理情况）
	xForwardedFor := r.Header.Get("X-Forwarded-For")
	if xForwardedFor != "" {
		// 取第一个IP
		ips := strings.Split(xForwardedFor, ",")
		if len(ips) > 0 {
			return strings.TrimSpace(ips[0])
		}
	}

	// 检查 X-Real-IP 头
	xRealIP := r.Header.Get("X-Real-IP")
	if xRealIP != "" {
		return xRealIP
	}

	// 从 RemoteAddr 获取
	ip := r.RemoteAddr
	// 去掉端口号
	if idx := strings.LastIndex(ip, ":"); idx != -1 {
		ip = ip[:idx]
	}

	return ip
}

// ServeWs WebSocket 服务端处理函数
func ServeWs(hub *Hub, w http.ResponseWriter, r *http.Request) {
	// 从 URL 参数获取客户端类型
	clientType := ClientType(r.URL.Query().Get("type"))
	if clientType != ClientTypeDesktop && clientType != ClientTypeMobile {
		clientType = ClientTypeMobile // 默认为手机端
	}

	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Println("WebSocket 升级失败:", err)
		return
	}

	client := Client{
		Type:        clientType,
		ID:          uuid.New().String(),
		UserAgent:   r.UserAgent(),
		IP:          getClientIP(r),
		ConnectedAt: time.Now(),
	}

	clientConn := NewClientConn(hub, conn, client)
	hub.register <- clientConn

	// 启动读写协程
	go clientConn.WritePump()
	go clientConn.ReadPump()
}
