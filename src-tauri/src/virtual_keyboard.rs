#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(windows)]
mod windows {
    use clipboard_win::set_clipboard_string;

    pub async fn paste_text(text: &str) -> Result<(), Box<dyn std::error::Error>> {
        set_clipboard_string(text)
            .map_err(|e| format!("设置剪贴板失败: {:?}", e))?;

        unsafe {
            use winapi::um::winuser::{keybd_event, VK_CONTROL};
            const VK_V: u8 = 0x56;

            keybd_event(VK_CONTROL as u8, 0, 0, 0);
            keybd_event(VK_V as u8, 0, 0, 0);
            keybd_event(VK_V as u8, 0, 2, 0);
            keybd_event(VK_CONTROL as u8, 0, 2, 0);
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux {
    pub async fn paste_text(text: &str) -> Result<(), Box<dyn std::error::Error>> {
        use clipboard::{ClipboardProvider, ClipboardContext};

        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
        ctx.set_contents(text.to_string())?;

        unsafe {
            use x11::xlib::{XOpenDisplay, XTestFakeKeyEvent, XFlush};

            let display = XOpenDisplay(std::ptr::null());
            if display.is_null() {
                return Err("无法打开 X Display".into());
            }

            let ctrl_key = 37u32;
            let v_key = 55u32;

            XTestFakeKeyEvent(display as _, ctrl_key, 1, 0);
            XTestFakeKeyEvent(display as _, v_key, 1, 0);
            XTestFakeKeyEvent(display as _, v_key, 0, 0);
            XTestFakeKeyEvent(display as _, ctrl_key, 0, 0);
            XFlush(display as _);
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    pub async fn paste_text(text: &str) -> Result<(), Box<dyn std::error::Error>> {
        use objc::{msg_send, sel, sel_impl};
        use cocoa::appkit::{NSPasteboard, NSStringPboardType};
        use cocoa::base::{id, nil};
        use cocoa::foundation::NSString;
        use core_foundation::base::TCFType;
        use core_graphics::event::{CGEvent, CGEventTapLocation};
        use core_graphics::keycode::KeyCode;

        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard(nil);
            let ns_string = NSString::alloc(nil).init_str(text);
            let _: () = msg_send![pasteboard, clearContents];
            let _: () = msg_send![pasteboard, setString:ns_string forType:NSStringPboardType];

            let kVK_Control = 0x3Bu8;
            let kVK_ANSI_V = 0x09u8;

            let control_down = CGEvent::new_keyboard_event(CGEventTapLocation::HID, kVK_Control as _, true);
            let v_down = CGEvent::new_keyboard_event(CGEventTapLocation::HID, kVK_ANSI_V as _, true);
            let v_up = CGEvent::new_keyboard_event(CGEventTapLocation::HID, kVK_ANSI_V as _, false);
            let control_up = CGEvent::new_keyboard_event(CGEventTapLocation::HID, kVK_Control as _, false);

            control_down.post(CGEventTapLocation::Session);
            v_down.post(CGEventTapLocation::Session);
            v_up.post(CGEventTapLocation::Session);
            control_up.post(CGEventTapLocation::Session);
        }

        Ok(())
    }
}
