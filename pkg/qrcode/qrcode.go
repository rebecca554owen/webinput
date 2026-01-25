package qrcode

import (
	"bytes"
	"image"

	"github.com/skip2/go-qrcode"
)

// GenerateQRCode 生成二维码
func GenerateQRCode(text string, size int) ([]byte, error) {
	img, err := qrcode.Encode(text, qrcode.Medium, size)
	if err != nil {
		return nil, err
	}

	return img, nil
}

// GenerateQRCodeImage 生成二维码图像对象
func GenerateQRCodeImage(text string, size int) (image.Image, error) {
	imgBytes, err := GenerateQRCode(text, size)
	if err != nil {
		return nil, err
	}

	img, _, err := image.Decode(bytes.NewReader(imgBytes))
	if err != nil {
		return nil, err
	}

	return img, nil
}
