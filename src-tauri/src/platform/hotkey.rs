use tauri::AppHandle;

#[cfg(target_os = "windows")]
pub fn start(app: AppHandle) {
    std::thread::Builder::new()
        .name("learnweave-global-hotkey".into())
        .spawn(move || unsafe {
            use windows::Win32::UI::Input::KeyboardAndMouse::{
                RegisterHotKey, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, VK_SPACE,
            };
            use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

            const HOTKEY_ID: i32 = 0x4c57;
            if RegisterHotKey(None, HOTKEY_ID, MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, VK_SPACE.0 as u32).is_err() {
                eprintln!("failed to register LearnWeave capture hotkey Ctrl+Shift+Space");
                return;
            }
            let mut message = MSG::default();
            while GetMessageW(&mut message, None, 0, 0).as_bool() {
                if message.message == WM_HOTKEY && message.wParam.0 == HOTKEY_ID as usize {
                    super::window::show_capture(&app);
                }
            }
        })
        .ok();
}

#[cfg(not(target_os = "windows"))]
pub fn start(_app: AppHandle) {}
