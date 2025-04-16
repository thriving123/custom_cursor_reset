// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod model;
pub mod utils;

use model::cursor_model::CursorInstallInfo;
use std::process::Command;

#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[tauri::command]
async fn get_device_info() -> model::cursor_model::CursorDeviceInfo {
    let package_path = utils::cursor_util::get_package_path();
    if !package_path.is_empty() {
        // 找到了包路径，读取设备信息
        return utils::cursor_util::read_device_info(package_path);
    }
    
    // 如果没有找到有效路径，返回默认值
    model::cursor_model::CursorDeviceInfo::new(
        "".to_owned(),
        "".to_owned(),
        "".to_owned(),
        "".to_owned(),
    )
}
#[tauri::command]
async fn reset_device_info() -> model::cursor_model::CursorDeviceInfo {
    let package_path = utils::cursor_util::get_package_path();
    if !package_path.is_empty() {
        return utils::cursor_util::reset_device_info(package_path);
    }
    model::cursor_model::CursorDeviceInfo::new(
        "".to_owned(),
        "".to_owned(),
        "".to_owned(),
        "".to_owned(),
    )
}


// 获取cursor安装路径 - 跨平台实现
#[tauri::command]
async fn get_cursor_install_info() -> CursorInstallInfo {
    let mut install_info = CursorInstallInfo::new("".to_owned(), "".to_owned(), "".to_owned(), "".to_owned());
        #[cfg(target_os = "windows")]
        {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let reg_path = r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{DADADADA-ADAD-ADAD-ADAD-ADADADADADAD}}_is1";
            if let Ok(subkey) = hkcu.open_subkey(reg_path) {
                if let Ok(install_path) = subkey.get_value::<String, _>("InstallLocation") {
                    install_info.install_path = install_path;
                }
                if let Ok(install_language) = subkey.get_value::<String, _>("Inno Setup: Language") {
                    install_info.install_language = install_language;
                }
                if let Ok(install_version) = subkey.get_value::<String, _>("DisplayVersion") {
                    install_info.install_version = install_version;
                }
                if let Ok(install_user) = subkey.get_value::<String, _>("Inno Setup: User") {
                    install_info.install_user = install_user;
                }
            }
            return install_info;
        }
        
        #[cfg(target_os = "macos")]
        {

            // macOS 上查找 Cursor 应用
            let applications = vec![
                "/Applications/Cursor.app".to_string(),
                format!("{}/Applications/Cursor.app", std::env::var("HOME").unwrap_or_default())
            ];
            
            for app_path in applications {
                if std::path::Path::new(&app_path).exists() {
                    install_info.install_path = app_path.to_string();
                    break;
                }
            }
            // 如果没找到，尝试使用 mdfind 命令查找
            if install_info.install_path.is_empty() {
                if let Ok(output) = Command::new("mdfind").args(["kMDItemCFBundleIdentifier == 'com.cursor.Cursor'"]).
                    output() {
                    if let Ok(stdout) = String::from_utf8(output.stdout) {
                        let lines: Vec<&str> = stdout.lines().collect();
                        if !lines.is_empty() {
                            install_info.install_path = lines[0].to_string();
                        }
                    }
                }
            }
            return install_info;
        }

}

// 重启 Cursor 应用
#[tauri::command]
async fn restart_cursor() -> bool {
    let cursor_path = get_cursor_install_info().await.install_path;
    if cursor_path.is_empty() {
        return false;
    }
    
    #[cfg(target_os = "windows")]
    {
        // 在 Windows 上，先尝试关闭 Cursor 进程
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "Cursor.exe"])
            .output();
        
        // 然后启动 Cursor
        if let Ok(_) = Command::new(&cursor_path).spawn() {
            return true;
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // 在 macOS 上，先尝试关闭 Cursor 进程
        let _ = Command::new("killall")
            .args(["Cursor"])
            .output();
        
        // 等待一秒确保进程完全关闭
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        // 如果路径以 .app 结尾，使用 open 命令
        if cursor_path.ends_with(".app") {
            if let Ok(_) = Command::new("open")
                .args(["-a", &cursor_path])
                .output() {
                return true;
            }
        } else {
            // 如果不是 .app 路径，尝试直接打开 Cursor.app
            if let Ok(_) = Command::new("open")
                .args(["-a", "Cursor"])
                .output() {
                return true;
            }
        }
    }
    
    false
}

// 检测 Cursor 是否正在运行
#[tauri::command]
async fn is_cursor_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        // 在 Windows 上检查进程
        if let Ok(output) = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq Cursor.exe", "/NH"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                return stdout.contains("Cursor.exe");
            }
        }
    }
    
    #[cfg(target_os = "macos")]
    {
        // 在 macOS 上检查进程
        if let Ok(output) = Command::new("pgrep")
            .args(["-x", "Cursor"])
            .output() {
            return !output.stdout.is_empty();
        }
    }
    
    false
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_device_info,
            get_cursor_install_info,
            reset_device_info,
            restart_cursor,
            is_cursor_running
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
