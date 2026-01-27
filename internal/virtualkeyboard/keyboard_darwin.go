//go:build darwin
// +build darwin

package virtualkeyboard

/*
#cgo LDFLAGS: -framework CoreGraphics

#include <ApplicationServices/ApplicationServices.h>

void sendPaste() {
    CGEventSourceRef source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);

    CGEventRef cmdDown = CGEventCreateKeyboardEvent(source, 55, true);
    CGEventSetFlags(cmdDown, kCGEventFlagMaskCommand);
    CGEventPost(kCGSessionEventTap, cmdDown);

    CGEventRef vDown = CGEventCreateKeyboardEvent(source, 9, true);
    CGEventSetFlags(vDown, kCGEventFlagMaskCommand);
    CGEventPost(kCGSessionEventTap, vDown);

    CGEventRef vUp = CGEventCreateKeyboardEvent(source, 9, false);
    CGEventPost(kCGSessionEventTap, vUp);

    CGEventRef cmdUp = CGEventCreateKeyboardEvent(source, 55, false);
    CGEventPost(kCGSessionEventTap, cmdUp);

    CFRelease(cmdDown);
    CFRelease(vDown);
    CFRelease(vUp);
    CFRelease(cmdUp);
    CFRelease(source);
}
*/
import "C"

import "time"

func SendCtrlV() error {
	C.sendPaste()
	time.Sleep(20 * time.Millisecond)
	return nil
}

func SendShiftInsert() error {
	return SendCtrlV()
}
