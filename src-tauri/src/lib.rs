// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod model;
pub mod utils;

use model::cursor_model::CursorInstallInfo;
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "windows")]
use winreg::enums::*;
#[cfg(target_os = "windows")]
use winreg::RegKey;

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
    let mut install_info =
        CursorInstallInfo::new("".to_owned(), "".to_owned(), "".to_owned(), "".to_owned());
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
            format!(
                "{}/Applications/Cursor.app",
                std::env::var("HOME").unwrap_or_default()
            ),
        ];

        for app_path in applications {
            if std::path::Path::new(&app_path).exists() {
                install_info.install_path = app_path.to_string();
                break;
            }
        }
        // 如果没找到，尝试使用 mdfind 命令查找
        if install_info.install_path.is_empty() {
            if let Ok(output) = Command::new("mdfind")
                .args(["kMDItemCFBundleIdentifier == 'com.cursor.Cursor'"])
                .output()
            {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    let lines: Vec<&str> = stdout.lines().collect();
                    if !lines.is_empty() {
                        install_info.install_path = lines[0].to_string();
                    }
                }
            }
        }

        // 获取版本信息
        if !install_info.install_path.is_empty() {
            // 从Info.plist获取版本信息
            let info_plist_path = format!("{}/Contents/Info.plist", install_info.install_path);
            if std::path::Path::new(&info_plist_path).exists() {
                // 使用defaults命令读取版本信息
                if let Ok(output) = Command::new("defaults")
                    .args(["read", &info_plist_path, "CFBundleShortVersionString"])
                    .output()
                {
                    if let Ok(version) = String::from_utf8(output.stdout) {
                        install_info.install_version = version.trim().to_string();
                    }
                }

                // 尝试多种方法获取Cursor的语言和用户信息
                let home_dir = std::env::var("HOME").unwrap_or_default();
                let cursor_config_dir = format!("{}/Library/Application Support/Cursor", home_dir);

                // 1. 尝试从Preferences文件获取语言设置
                let preferences_path = format!("{}/Preferences", cursor_config_dir);
                if std::path::Path::new(&preferences_path).exists() {
                    if let Ok(output) = Command::new("cat").arg(&preferences_path).output() {
                        if let Ok(prefs_content) = String::from_utf8(output.stdout) {
                            if let Some(lang_pos) = prefs_content.find("dictionaries\":") {
                                let lang_start = lang_pos + 14; // dictionaries":[" 的长度
                                if let Some(lang_end) = prefs_content[lang_start..].find("]") {
                                    let language =
                                        &prefs_content[lang_start..lang_start + lang_end];
                                    // 处理引号和逗号
                                    let cleaned_lang = language
                                        .replace('"', "")
                                        .replace('[', "")
                                        .trim()
                                        .to_string();
                                    if !cleaned_lang.is_empty() {
                                        install_info.install_language = cleaned_lang;
                                    }
                                }
                            }
                        }
                    }
                }

                // 2. 尝试从Git日志中获取用户名和邮箱
                let logs_dir = format!("{}/logs", cursor_config_dir);
                if std::path::Path::new(&logs_dir).exists() {
                    // 使用grep命令从日志中查找用户名和邮箱
                    if let Ok(output) = Command::new("grep")
                        .args(["-r", "Stored git author name", &logs_dir])
                        .output()
                    {
                        if let Ok(log_content) = String::from_utf8(output.stdout) {
                            if !log_content.is_empty() {
                                // 从日志中提取用户名和邮箱
                                if let Some(author_pos) = log_content.find("global state: ") {
                                    let author_start = author_pos + 14; // "global state: " 的长度
                                    let author_info = &log_content[author_start..];

                                    // 如果包含用户名和邮箱的格式如"李良安 <1120777912@qq.com>"
                                    if let Some(email_start) = author_info.find('<') {
                                        let username = author_info[..email_start].trim();
                                        if let Some(email_end) =
                                            author_info[email_start..].find('>')
                                        {
                                            let email = &author_info
                                                [email_start + 1..email_start + email_end];
                                            if !username.is_empty() && !email.is_empty() {
                                                // 同时显示用户名和邮箱，格式为"用户名(邮箱)"
                                                install_info.install_user =
                                                    format!("{} ({})", username, email);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 3. 如果从Git日志中没有获取到用户名，尝试从sentry/scope_v3.json获取邮箱
                if install_info.install_user.is_empty() {
                    let sentry_path = format!("{}/sentry/scope_v3.json", cursor_config_dir);
                    if std::path::Path::new(&sentry_path).exists() {
                        if let Ok(output) = Command::new("cat").arg(&sentry_path).output() {
                            if let Ok(sentry_content) = String::from_utf8(output.stdout) {
                                // 尝试获取用户邮箱
                                if let Some(email_pos) = sentry_content.find("\"email\":") {
                                    let email_start = email_pos + 9; // "email":" 的长度
                                    if let Some(email_end) =
                                        sentry_content[email_start..].find("\"")
                                    {
                                        let email =
                                            &sentry_content[email_start..email_start + email_end];
                                        if !email.is_empty() {
                                            install_info.install_user = format!("用户({})", email);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // 3. 尝试从languagepacks.json获取语言设置
                if install_info.install_language.is_empty() {
                    let lang_packs_path = format!("{}/languagepacks.json", cursor_config_dir);
                    if std::path::Path::new(&lang_packs_path).exists() {
                        if let Ok(output) = Command::new("cat").arg(&lang_packs_path).output() {
                            if let Ok(lang_content) = String::from_utf8(output.stdout) {
                                // 如果文件包含中文语言包
                                if lang_content.contains("zh-cn") {
                                    install_info.install_language = "zh-cn".to_string();
                                } else if lang_content.contains("en") {
                                    install_info.install_language = "en".to_string();
                                }
                            }
                        }
                    }
                }

                // 4. 检查中文语言包目录是否存在
                if install_info.install_language.is_empty() {
                    let zh_lang_dir = format!(
                        "{}/clp/6d6cd612ec0ae3cd32737a6f6b7ad966.zh-cn",
                        cursor_config_dir
                    );
                    if std::path::Path::new(&zh_lang_dir).exists() {
                        install_info.install_language = "zh-cn".to_string();
                    }
                }

                // 5. 如果还是没有获取到用户信息，尝试使用应用程序路径的用户名部分
                if install_info.install_user.is_empty() && !install_info.install_path.is_empty() {
                    if install_info.install_path.contains("/Users/") {
                        if let Some(user_start) = install_info.install_path.find("/Users/") {
                            let user_path = &install_info.install_path[user_start + 7..];
                            if let Some(user_end) = user_path.find('/') {
                                let username = &user_path[..user_end];
                                if !username.is_empty() {
                                    install_info.install_user = username.to_string();
                                }
                            }
                        }
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
    let cursor_path = get_cursor_install_info().await.install_path + "cursor.exe";
    if cursor_path.is_empty() {
        return false;
    }

    #[cfg(target_os = "windows")]
    {
        // 在 Windows 上，先尝试关闭 Cursor 进程
        let _ = Command::new("taskkill")
            .args(["/F", "/IM", "Cursor.exe"])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .output();

        // 然后启动 Cursor
        if let Ok(_) = Command::new(&cursor_path)
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn()
        {
            return true;
        }
    }

    #[cfg(target_os = "macos")]
    {
        // 在 macOS 上，先尝试关闭 Cursor 进程
        let _ = Command::new("killall").args(["Cursor"]).output();

        // 等待一秒确保进程完全关闭
        std::thread::sleep(std::time::Duration::from_secs(1));

        // 如果路径以 .app 结尾，使用 open 命令
        if cursor_path.ends_with(".app") {
            if let Ok(_) = Command::new("open").args(["-a", &cursor_path]).output() {
                return true;
            }
        } else {
            // 如果不是 .app 路径，尝试直接打开 Cursor.app
            if let Ok(_) = Command::new("open").args(["-a", "Cursor"]).output() {
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
            .output()
        {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                return stdout.contains("Cursor.exe");
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // 在 macOS 上检查进程
        if let Ok(output) = Command::new("pgrep").args(["-x", "Cursor"]).output() {
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
