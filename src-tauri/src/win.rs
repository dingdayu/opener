use std::ptr;
use winapi::um::winuser::{SetForegroundWindow, IsWindowVisible, GetClassNameA};
use winapi::shared::windef::HWND;
use windows::{
    Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, CoCreateInstance, CLSCTX_ALL},
    Win32::UI::Shell::{IShellWindows, IShellFolderViewDual, ShellWindows},
    Win32::System::Variant::{VARIANT, VT_I4},
};

/// 遍历所有窗口，获取文件资源管理器的 HWND
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: isize) -> i32 {
    let windows: &mut Vec<HWND> = &mut *(lparam as *mut Vec<HWND>);
    let mut class_name: [u8; 256] = [0; 256];

    // 获取窗口类名
    let class_length = GetClassNameA(hwnd, class_name.as_mut_ptr() as *mut i8, class_name.len() as i32);
    let class_name = String::from_utf8_lossy(&class_name[..class_length as usize]);

    // 判断是否是 Explorer 窗口，并且是可见窗口
    if class_name.contains("CabinetWClass") && IsWindowVisible(hwnd) != 0 {
        windows.push(hwnd);
    }
    1 // 继续枚举
}

/// 获取所有 Explorer 窗口的路径
fn get_explorer_windows() -> Vec<(HWND, String)> {
    let mut result = Vec::new();
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
        let shell_windows: IShellWindows = CoCreateInstance::<ShellWindows>(CLSCTX_ALL).unwrap();

        for i in 0..shell_windows.Count().unwrap() {
            let mut variant = VARIANT::default();
            variant.Anonymous.Anonymous.vt = VT_I4;
            variant.Anonymous.Anonymous.Anonymous.lVal = i as i32;

            if let Ok(window) = shell_windows.Item(variant) {
                if let Ok(folder_view) = window.Query::<IShellFolderViewDual>() {
                    if let Ok(path) = folder_view.Folder().unwrap().Self_().unwrap().Name() {
                        let hwnd = window.HWND().unwrap();
                        result.push((hwnd as HWND, path.to_string()));
                    }
                }
            }
        }
        CoUninitialize();
    }
    result
}

/// 设置文件资源管理器窗口置顶（匹配 `open_path` 传入的路径）
pub fn set_window_topmost(target_path: &str) { // ✅ `pub` 使其可在 `lib.rs` 访问
    let explorer_windows = get_explorer_windows();

    for (hwnd, path) in explorer_windows {
        if path.to_lowercase() == target_path.to_lowercase() {
            unsafe {
                SetForegroundWindow(hwnd);
            }
            break;
        }
    }
}
