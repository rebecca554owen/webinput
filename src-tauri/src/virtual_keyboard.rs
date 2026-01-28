#[cfg(windows)]
pub use windows::*;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(windows)]
mod windows {
    use clipboard_win::set_clipboard_string;

    pub async fn paste_text(text: &str, append_enter: bool) -> Result<(), Box<dyn std::error::Error>> {
        set_clipboard_string(text)
            .map_err(|e| format!("设置剪贴板失败: {:?}", e))?;

        unsafe {
            use winapi::um::winuser::{keybd_event, VK_CONTROL, VK_RETURN};
            const VK_V: u8 = 0x56;

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
    pub async fn paste_text(text: &str, append_enter: bool) -> Result<(), Box<dyn std::error::Error>> {
        use clipboard::{ClipboardProvider, ClipboardContext};

        let mut ctx: ClipboardContext = ClipboardProvider::new()?;
        ctx.set_contents(text.to_string())?;

        unsafe {
            use x11::xlib::{XOpenDisplay, XFlush};
            use x11::xtest::XTestFakeKeyEvent;

            let display = XOpenDisplay(std::ptr::null());
            if display.is_null() {
                return Err("无法打开 X Display".into());
            }

            let ctrl_key = 37u32;
            let v_key = 55u32;
            let return_key = 36u32;

            XTestFakeKeyEvent(display as _, ctrl_key, 1, 0);
            XTestFakeKeyEvent(display as _, v_key, 1, 0);
            XTestFakeKeyEvent(display as _, v_key, 0, 0);
            XTestFakeKeyEvent(display as _, ctrl_key, 0, 0);

            if append_enter {
                XTestFakeKeyEvent(display as _, return_key, 1, 0);
                XTestFakeKeyEvent(display as _, return_key, 0, 0);
            }

            XFlush(display as _);
        }

        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    pub async fn paste_text(text: &str, append_enter: bool) -> Result<(), Box<dyn std::error::Error>> {
        use objc::{msg_send, sel, sel_impl};
        use cocoa::appkit::{NSPasteboard, NSStringPboardType};
        use cocoa::base::nil;
        use cocoa::foundation::NSString;
        use core_graphics::event::{CGEvent, CGEventTapLocation};
        use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

        unsafe {
            let pasteboard = NSPasteboard::generalPasteboard(nil);
            let ns_string = NSString::alloc(nil).init_str(text);
            let _: () = msg_send![pasteboard, clearContents];
            let _: () = msg_send![pasteboard, setString:ns_string forType:NSStringPboardType];

            let kVK_Control = 0x3Bu8;
            let kVK_ANSI_V = 0x09u8;
            let kVK_Return = 0x24u8;

            let source = CGEventSource::new(CGEventSourceStateID::Private).expect("Failed to create event source");

            let control_down = CGEvent::new_keyboard_event(source.clone(), kVK_Control as _, true).expect("Failed to create Control down event");
            let v_down = CGEvent::new_keyboard_event(source.clone(), kVK_ANSI_V as _, true).expect("Failed to create V down event");
            let v_up = CGEvent::new_keyboard_event(source.clone(), kVK_ANSI_V as _, false).expect("Failed to create V up event");
            let control_up = CGEvent::new_keyboard_event(source.clone(), kVK_Control as _, false).expect("Failed to create Control up event");

            control_down.post(CGEventTapLocation::Session);
            v_down.post(CGEventTapLocation::Session);
            v_up.post(CGEventTapLocation::Session);
            control_up.post(CGEventTapLocation::Session);

            if append_enter {
                let return_down = CGEvent::new_keyboard_event(source.clone(), kVK_Return as _, true).expect("Failed to create Return down event");
                let return_up = CGEvent::new_keyboard_event(source, kVK_Return as _, false).expect("Failed to create Return up event");
                return_down.post(CGEventTapLocation::Session);
                return_up.post(CGEventTapLocation::Session);
            }
        }

        Ok(())
    }
}
