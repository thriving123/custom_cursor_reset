#[derive(Clone, serde::Serialize)]
pub struct CursorDeviceInfo {
    pub mac_machine_id: String,
    pub machine_id: String,
    pub sqm_id: String,
    pub dev_device_id: String,
}

impl CursorDeviceInfo {
    pub fn new(mac_machine_id: String,
               machine_id: String,
               sqm_id: String,
               dev_device_id: String) -> CursorDeviceInfo {
        return CursorDeviceInfo { mac_machine_id, machine_id, sqm_id, dev_device_id };
    }

    pub fn reset(self)-> CursorDeviceInfo {
        return self
    }
    pub fn blocking_kind(&self) -> String {
        // 方法实现
        return format!("{}:{}:{}:{}", self.mac_machine_id.clone(), self.machine_id.clone(), self.sqm_id.clone(), self.dev_device_id.clone());
    }
}

#[derive(Clone, serde::Serialize)]
pub struct CursorInstallInfo {
    pub install_path: String,
    pub install_language: String,
    pub install_version: String,
    pub install_user: String,
}

impl CursorInstallInfo {
    pub fn new(install_path: String, install_language: String, install_version: String, install_user: String) -> Self {
        Self { install_path, install_language, install_version, install_user }
    }
}