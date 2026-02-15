#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(windows)]
mod windows {
    use clipboard_win::set_clipboard_string;

    // Windows 虚拟键码常量
    const VK_V: u8 = 0x56;

    pub async fn paste_text(text: &str, append_enter: bool) -> Result<(), Box<dyn std::error::Error>> {
        set_clipboard_string(text)
            .map_err(|e| format!("设置剪贴板失败: {:?}", e))?;

        unsafe {
            use winapi::um::winuser::{keybd_event, VK_CONTROL, VK_RETURN};

            keybd_event(VK_CONTROL as u8, 0, 0, 0);
            keybd_event(VK_V, 0, 0, 0);
            keybd_event(VK_V, 0, 2, 0);
            keybd_event(VK_CONTROL as u8, 0, 2, 0);

            if append_enter {
                keybd_event(VK_RETURN as u8, 0, 0, 0);
                keybd_event(VK_RETURN as u8, 0, 2, 0);
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use x11::xlib::{XCloseDisplay, XOpenDisplay, XFlush};
    use x11::xtest::XTestFakeKeyEvent;

    // X11 按键码常量
    const CTRL_KEY: u32 = 37;
    const V_KEY: u32 = 55;
    const RETURN_KEY: u32 = 36;

    pub async fn paste_text(text: &str, append_enter: bool) -> Result<(), Box<dyn std::error::Error>> {
        use clipboard::{ClipboardProvider, ClipboardContext};

        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
        ctx.set_contents(text.to_string())?;

        unsafe {
            let display = XOpenDisplay(std::ptr::null());
            if display.is_null() {
                return Err("无法打开 X Display".into());
            }

            XTestFakeKeyEvent(display as _, CTRL_KEY, 1, 0);
            XTestFakeKeyEvent(display as _, V_KEY, 1, 0);
            XTestFakeKeyEvent(display as _, V_KEY, 0, 0);
            XTestFakeKeyEvent(display as _, CTRL_KEY, 0, 0);

            if append_enter {
                XTestFakeKeyEvent(display as _, RETURN_KEY, 1, 0);
                XTestFakeKeyEvent(display as _, RETURN_KEY, 0, 0);
            }

            XFlush(display as _);
            XCloseDisplay(display);
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use objc::{msg_send, sel, sel_impl};
    use cocoa::appkit::{NSPasteboard, NSStringPboardType};
    use cocoa::base::nil;
    use cocoa::foundation::NSString;
    use core_graphics::event::{CGEvent, CGEventTapLocation};
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    // macOS 虚拟键码常量
    const KV_CONTROL: u8 = 0x3B;
    const KV_ANSI_V: u8 = 0x09;
    const KV_RETURN: u8 = 0x24;

    pub async fn paste_text(text: &str, append_enter: bool) -> Result<(), Box<dyn std::error::Error>> {
        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard(nil);
            let ns_string = NSString::alloc(nil).init_str(text);
            let _: () = msg_send![pasteboard, clearContents];
            let _: () = msg_send![pasteboard, setString:ns_string forType:NSStringPboardType];

            let source = CGEventSource::new(CGEventSourceStateID::Private).expect("Failed to create event source");

            let control_down = CGEvent::new_keyboard_event(source.clone(), KV_CONTROL as _, true).expect("Failed to create Control down event");
            let v_down = CGEvent::new_keyboard_event(source.clone(), KV_ANSI_V as _, true).expect("Failed to create V down event");
            let v_up = CGEvent::new_keyboard_event(source.clone(), KV_ANSI_V as _, false).expect("Failed to create V up event");
            let control_up = CGEvent::new_keyboard_event(source.clone(), KV_CONTROL as _, false).expect("Failed to create Control up event");

            control_down.post(CGEventTapLocation::Session);
            v_down.post(CGEventTapLocation::Session);
            v_up.post(CGEventTapLocation::Session);
            control_up.post(CGEventTapLocation::Session);

            if append_enter {
                let return_down = CGEvent::new_keyboard_event(source.clone(), KV_RETURN as _, true).expect("Failed to create Return down event");
                let return_up = CGEvent::new_keyboard_event(source, KV_RETURN as _, false).expect("Failed to create Return up event");
                return_down.post(CGEventTapLocation::Session);
                return_up.post(CGEventTapLocation::Session);
            }
        }

        Ok(())
    }
}
