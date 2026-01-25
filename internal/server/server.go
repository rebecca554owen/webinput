package server

import (
	_ "embed"
	"encoding/json"
	"net/http"
	"strconv"
	"time"

	"github.com/rebecca554owen/webinput/internal/config"
	"github.com/rebecca554owen/webinput/internal/logger"
	"github.com/rebecca554owen/webinput/internal/virtualkeyboard"
	"github.com/rebecca554owen/webinput/internal/websocket"
)

//go:embed assets/mobile.html
var mobileHTML []byte

// Server HTTP服务器实例
type Server struct {
	config  *config.Config
	mux     *http.ServeMux
	server  *http.Server
	errChan chan error    // 错误通道
	started chan struct{} // 启动完成信号
	hub     *websocket.Hub // WebSocket Hub
}

// NewServer 创建新的服务器实例
func NewServer(config *config.Config) *Server {
	s := &Server{
		config: config,
		mux:    http.NewServeMux(),
		hub:    websocket.NewHub(), // 初始化 Hub
	}

	// 启动 Hub
	go s.hub.Run()

	// 注册路由
	s.registerRoutes()

	return s
}

// registerRoutes 注册HTTP路由
func (s *Server) registerRoutes() {
	// 主页
	s.mux.HandleFunc("/", s.homeHandler)
	// 输入文本接口
	s.mux.HandleFunc("/type", s.typeHandler)
	// WebSocket 接口
	s.mux.HandleFunc("/ws", s.websocketHandler)
	// 静态文件服务
	s.mux.Handle("/web/", http.StripPrefix("/web/", http.FileServer(http.Dir("./web"))))
}

// homeHandler 主页处理函数
func (s *Server) homeHandler(w http.ResponseWriter, r *http.Request) {
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	w.Write(mobileHTML)
}

// websocketHandler WebSocket 处理函数
func (s *Server) websocketHandler(w http.ResponseWriter, r *http.Request) {
	websocket.ServeWs(s.hub, w, r)
}

// typeHandler 输入文本接口处理函数
func (s *Server) typeHandler(w http.ResponseWriter, r *http.Request) {
	if r.Method != http.MethodPost {
		w.WriteHeader(http.StatusMethodNotAllowed)
		return
	}

	// 解析请求数据
	var req struct {
		Text string `json:"text"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		logger.Error("解析请求数据失败: " + err.Error())
		w.WriteHeader(http.StatusBadRequest)
		json.NewEncoder(w).Encode(map[string]bool{"success": false})
		return
	}

	// 发送文本到虚拟键盘
	if err := virtualkeyboard.PasteText(req.Text); err != nil {
		logger.Error("发送文本失败: " + err.Error())
		w.WriteHeader(http.StatusInternalServerError)
		json.NewEncoder(w).Encode(map[string]bool{"success": false})
		return
	}

	logger.Info("文本发送成功: " + req.Text)
	json.NewEncoder(w).Encode(map[string]bool{"success": true})
}

// Start 启动服务器（阻塞等待启动结果）
func (s *Server) Start() error {
	port, err := strconv.Atoi(s.config.Port)
	if err != nil {
		return err
	}

	addr := ":" + strconv.Itoa(port)
	s.server = &http.Server{
		Addr:    addr,
		Handler: s.mux,
	}

	// 初始化通道
	s.errChan = make(chan error, 1)
	s.started = make(chan struct{})

	// 在 goroutine 中启动服务器
	go func() {
		err := s.server.ListenAndServe()
		select {
		case <-s.started:
			// 已启动成功，这是运行时错误（Close）
			s.errChan <- err
		default:
			// 启动阶段失败（端口被占用等）
			s.errChan <- err
		}
	}()

	// 短暂等待端口绑定完成
	time.Sleep(100 * time.Millisecond)

	select {
	case err := <-s.errChan:
		// 启动失败
		return err
	default:
		// 假设启动成功
		close(s.started)
		return nil
	}
}

// Stop 停止服务器
func (s *Server) Stop() error {
	if s.server != nil {
		return s.server.Close()
	}
	return nil
}
