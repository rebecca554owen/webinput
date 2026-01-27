//go:build linux
// +build linux

package virtualkeyboard

/*
#cgo LDFLAGS: -lX11 -lXtst

#include <X11/Xlib.h>
#include <X11/keysym.h>
#include <X11/extensions/XTest.h>

Display *display = NULL;

void initDisplay() {
    if (display == NULL) {
        display = XOpenDisplay(NULL);
    }
}

void closeDisplay() {
    if (display != NULL) {
        XCloseDisplay(display);
        display = NULL;
    }
}

void sendCtrlV() {
    initDisplay();
    if (display == NULL) return;

    KeyCode ctrl = XKeysymToKeycode(display, XK_Control_L);
    KeyCode v = XKeysymToKeycode(display, XK_v);

    XTestFakeKeyEvent(display, ctrl, True, CurrentTime);
    XTestFakeKeyEvent(display, v, True, CurrentTime);
    XTestFakeKeyEvent(display, v, False, CurrentTime);
    XTestFakeKeyEvent(display, ctrl, False, CurrentTime);
    XFlush(display);
}

void sendShiftInsert() {
    initDisplay();
    if (display == NULL) return;

    KeyCode shift = XKeysymToKeycode(display, XK_Shift_L);
    KeyCode insert = XKeysymToKeycode(display, XK_Insert);

    XTestFakeKeyEvent(display, shift, True, CurrentTime);
    XTestFakeKeyEvent(display, insert, True, CurrentTime);
    XTestFakeKeyEvent(display, insert, False, CurrentTime);
    XTestFakeKeyEvent(display, shift, False, CurrentTime);
    XFlush(display);
}
*/
import "C"

import (
	"time"
)

func SendCtrlV() error {
	C.sendCtrlV()
	time.Sleep(20 * time.Millisecond)
	return nil
}

func SendShiftInsert() error {
	C.sendShiftInsert()
	time.Sleep(20 * time.Millisecond)
	return nil
}
