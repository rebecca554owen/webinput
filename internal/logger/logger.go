package logger

import (
	"log"
	"os"
)

// Logger 日志接口
type Logger interface {
	Info(msg string)
	Error(msg string)
	Debug(msg string)
}

// SimpleLogger 简单日志实现
type SimpleLogger struct {
	infoLogger  *log.Logger
	errorLogger *log.Logger
	debugLogger *log.Logger
}

// NewSimpleLogger 创建新的简单日志实例
func NewSimpleLogger() *SimpleLogger {
	return &SimpleLogger{
		infoLogger:  log.New(os.Stdout, "[INFO] ", log.Ldate|log.Ltime),
		errorLogger: log.New(os.Stderr, "[ERROR] ", log.Ldate|log.Ltime|log.Lshortfile),
		debugLogger: log.New(os.Stdout, "[DEBUG] ", log.Ldate|log.Ltime|log.Lshortfile),
	}
}

// Info 记录信息级日志
func (l *SimpleLogger) Info(msg string) {
	l.infoLogger.Println(msg)
}

// Error 记录错误级日志
func (l *SimpleLogger) Error(msg string) {
	l.errorLogger.Println(msg)
}

// Debug 记录调试级日志
func (l *SimpleLogger) Debug(msg string) {
	l.debugLogger.Println(msg)
}

// 全局日志实例
var globalLogger = NewSimpleLogger()

// Info 记录信息级日志（全局函数）
func Info(msg string) {
	globalLogger.Info(msg)
}

// Error 记录错误级日志（全局函数）
func Error(msg string) {
	globalLogger.Error(msg)
}

// Debug 记录调试级日志（全局函数）
func Debug(msg string) {
	globalLogger.Debug(msg)
}
