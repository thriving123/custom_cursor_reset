#[cfg(target_os = "windows")]
const PACKAGE_JSON_ARR: [&str; 3] = [
    "AppData\\Roaming\\Cursor\\User\\globalStorage\\storage.json",
    "AppData\\Programs\\cursor\\resources\\app\\package.json",
    "AppData\\cursor\\resources\\app\\package.json",
];

#[cfg(target_os = "macos")]
const PACKAGE_JSON_ARR: [&str; 3] = [
    "Library/Application Support/Cursor/User/globalStorage/storage.json",
    "Applications/Cursor.app/Contents/Resources/app/package.json",
    "Library/Application Support/Cursor/storage.json",
];

use std::env;
use std::path::PathBuf;
use std::fs::File;
use std::io::{Read, Write};
use serde_json::Value;
use crate::model::cursor_model::CursorDeviceInfo;
use uuid::Uuid;
use rand::Rng;

// 返回当前用户根目录
fn get_user_home_dir() -> String {
  #[cfg(target_os = "windows")]
  {
    match env::var_os("USERPROFILE") {
      Some(home) => PathBuf::from(home).to_string_lossy().to_string(),
      None => String::from("") // 如果没有找到，返回空字符串
    }
  }
  
  #[cfg(not(target_os = "windows"))]
  {
    match env::var_os("HOME") {
      Some(home) => PathBuf::from(home).to_string_lossy().to_string(),
      None => String::from("") // 如果没有找到，返回空字符串
    }
  }
}

pub fn get_package_path() -> String {
  let home_dir = get_user_home_dir();
  if home_dir.is_empty() {
    return "".to_string();
  }
  
  for item in PACKAGE_JSON_ARR {
    #[cfg(target_os = "windows")]
    let full_path = format!("{}\\{}", home_dir, item);
    
    #[cfg(not(target_os = "windows"))]
    let full_path = format!("{}/{}", home_dir, item);
    
    if std::path::Path::new(&full_path).exists() {
      return full_path;
    }
  }
  
  // 对于 macOS，尝试直接在系统级别查找
  #[cfg(target_os = "macos")]
  {
    let system_paths = [
      "/Applications/Cursor.app/Contents/Resources/app/package.json",
      "/Applications/Cursor.app/Contents/Resources/storage.json"
    ];
    
    for path in system_paths.iter() {
      if std::path::Path::new(path).exists() {
        return path.to_string();
      }
    }
  }
  
  "".to_string()
}

pub fn read_device_info(path: String) -> CursorDeviceInfo {
  // 默认值
  let mut mac_machine_id = "".to_string();
  let mut machine_id = "".to_string();
  let mut sqm_id = "".to_string();
  let mut dev_device_id = "".to_string();
  
  // 读取文件
  let file = File::open(&path);
  if let Ok(mut file) = file {
    let mut contents = String::new();
    if file.read_to_string(&mut contents).is_ok() {
      // 解析 JSON
      if let Ok(json) = serde_json::from_str::<Value>(&contents) {
        // 作为 Map 迭代所有键值对
        if let Some(obj) = json.as_object() {
          for (key, value) in obj {
            match key.as_str() {
              "telemetry.macMachineId" => {
                if let Some(str_val) = value.as_str() {
                  mac_machine_id = str_val.to_string();
                }
              },
              "telemetry.machineId" => {
                if let Some(str_val) = value.as_str() {
                  machine_id = str_val.to_string();
                }
              },
              "telemetry.sqmId" => {
                if let Some(str_val) = value.as_str() {
                  sqm_id = str_val.to_string();
                }
              },
              "telemetry.devDeviceId" => {
                if let Some(str_val) = value.as_str() {
                  dev_device_id = str_val.to_string();
                }
              },
              _ => {} // 忽略其他键
            }
          }
        }
      }
    }
  }
  
  // 创建并返回 CursorDeviceInfo 实例
  CursorDeviceInfo::new(mac_machine_id, machine_id, sqm_id, dev_device_id)
}

fn generate_hex_str(size:i32)->String{
    let mut rng = rand::rng();
    let mut result = String::with_capacity(size as usize);
    for _ in 0..size {
        let random_digit: u8 = rng.random_range(0..16);
        let hex_char = format!("{:x}", random_digit);
        result.push_str(&hex_char);
    }
    result
}

pub fn reset_device_info(path:String) -> CursorDeviceInfo {
    let mac_machine_id = Uuid::new_v4().to_string();
    let machine_id = generate_hex_str(64);
    let sqm_id = format!("{{{}}}", Uuid::new_v4().to_string().to_uppercase());
    let dev_device_id = Uuid::new_v4().to_string();
    let info = CursorDeviceInfo::new(mac_machine_id, machine_id, sqm_id, dev_device_id);
    // 写到文件
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();

    let mut json_map = serde_json::Map::new();
    json_map.insert("telemetry.macMachineId".to_string(), serde_json::Value::String(info.mac_machine_id.clone()));
    json_map.insert("telemetry.machineId".to_string(), serde_json::Value::String(info.machine_id.clone()));
    json_map.insert("telemetry.sqmId".to_string(), serde_json::Value::String(info.sqm_id.clone()));
    json_map.insert("telemetry.devDeviceId".to_string(), serde_json::Value::String(info.dev_device_id.clone()));

    let json_str = serde_json::to_string_pretty(&json_map).unwrap();
    file.write_all(json_str.as_bytes()).unwrap();
    info
}