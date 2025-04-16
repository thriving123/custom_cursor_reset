<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { Message } from "becomer-ui";
// 设备信息
interface IDeviceInfo {
  mac_machine_id: string;
  machine_id: string;
  sqm_id: string;
  dev_device_id: string;
}

interface IInstallInfo {
  install_path: string;
  install_language: string;
  install_version: string;
  install_user: string;
}

const deviceInfo = ref<IDeviceInfo>();
const get_data = async () => {
  deviceInfo.value = await invoke("get_device_info");
};
const reset_device_info = async () => {
  try {
    deviceInfo.value = await invoke("reset_device_info");
    Message.success("机器码重置成功");
  } catch (error) {
    Message.error(`机器码重置失败: ${error}`);
  }
};
// 安装信息
const installInfo = ref<IInstallInfo>();
const get_install = async () => {
  installInfo.value = await invoke("get_cursor_install_info");
};

// 重启 Cursor
const isRestarting = ref(false);
const restart_cursor = async () => {
  try {
    isRestarting.value = true;
    const result = await invoke("restart_cursor");
    if (result) {
      Message.success("Cursor 已重启");
      // 重启后等待一下再检查状态
      setTimeout(check_cursor_status, 2000);
    } else {
      Message.error("重启 Cursor 失败，请检查 Cursor 是否已安装");
    }
  } catch (error) {
    Message.error(`重启失败: ${error}`);
  } finally {
    isRestarting.value = false;
  }
};

// 检测 Cursor 运行状态
const cursorRunning = ref(false);

const check_cursor_status = async () => {
  cursorRunning.value = await invoke("is_cursor_running");
};

// 定时检测运行状态
let statusInterval: number | null = null;

onMounted(() => {
  // 获取安装位置
  get_install();
  get_data();

  // 初始检测运行状态
  check_cursor_status();

  // 每 5 秒检测一次运行状态
  statusInterval = setInterval(check_cursor_status, 1000) as unknown as number;
});

// 组件卸载时清除定时器
onUnmounted(() => {
  if (statusInterval !== null) {
    clearInterval(statusInterval);
  }
});
</script>

<template>
  <main class="container">
    <b-container>
      <b-header height="30">
        <b-grid>
          <b-grid-item column="12">
            <h3>Cursor 重置工具</h3>
          </b-grid-item>
          <b-grid-item offset="-1" column="1">
            <b-tag v-if="cursorRunning" type="success">正在运行</b-tag>
            <b-tag v-else type="danger">未运行</b-tag>
          </b-grid-item>
        </b-grid>
      </b-header>
      <b-main class="main-content">
        <b-grid gap="20" style="height: calc(100% - 10px)">
          <b-grid-item column="12" style="width: 100%">
            <b-card header="安装信息" column="12" style="height: 100%">
              <b-grid>
                <b-grid-item column="6"
                  ><p class="info-label">安装地址</p></b-grid-item
                >
                <b-grid-item column="18"
                  ><b-tag type="warning">{{ installInfo?.install_path }}</b-tag></b-grid-item
                >
              </b-grid>
              <b-divider size="small"></b-divider>
              <b-grid>
                <b-grid-item column="6"
                  ><p class="info-label">安装语言</p></b-grid-item
                >
                <b-grid-item column="18"
                  ><b-tag type="warning">{{ installInfo?.install_language }}</b-tag></b-grid-item
                >
              </b-grid>
              <b-divider size="small"></b-divider>
              <b-grid>
                <b-grid-item column="6"
                  ><p class="info-label">安装版本</p></b-grid-item
                >
                <b-grid-item column="18"
                  ><b-tag type="warning">{{ installInfo?.install_version }}</b-tag></b-grid-item
                >
              </b-grid>
              <b-divider size="small"></b-divider>
              <b-grid>
                <b-grid-item column="6"
                  ><p class="info-label">安装用户</p></b-grid-item
                >
                <b-grid-item column="18"
                  ><b-tag type="warning">{{ installInfo?.install_user }}</b-tag></b-grid-item
                >
              </b-grid>

            </b-card>
          </b-grid-item>
          <b-grid-item column="12" style="width: 100%">
            <b-card header="机器码信息" column="12" style="height: 100%">
              <b-grid>
                <b-grid-item column="6"
                  ><p class="info-label">mac_machine_id</p></b-grid-item
                >
                <b-grid-item column="18"
                  ><b-tag type="primary">{{
                    deviceInfo?.mac_machine_id
                  }}</b-tag></b-grid-item
                >
              </b-grid>
              <b-divider size="small"></b-divider>
              <b-grid>
                <b-grid-item column="6"
                  ><p class="info-label">machine_id</p></b-grid-item
                >
                <b-grid-item column="18"
                  ><b-tag type="primary">{{
                    deviceInfo?.machine_id
                  }}</b-tag></b-grid-item
                >
              </b-grid>
              <b-divider size="small"></b-divider>
              <b-grid>
                <b-grid-item column="6"
                  ><p class="info-label">sqm_id</p></b-grid-item
                >
                <b-grid-item column="18"
                  ><b-tag type="primary">{{
                    deviceInfo?.sqm_id
                  }}</b-tag></b-grid-item
                >
              </b-grid>
              <b-divider size="small"></b-divider>
              <b-grid>
                <b-grid-item column="6"
                  ><p class="info-label">dev_device_id</p></b-grid-item
                >
                <b-grid-item column="18"
                  ><b-tag type="primary">{{
                    deviceInfo?.dev_device_id
                  }}</b-tag></b-grid-item
                >
              </b-grid>
            </b-card>
          </b-grid-item>
        </b-grid>
      </b-main>
      <b-footer height="85">
        <b-grid gap="20" class="footer-actions">
          <b-grid-item column="8">
            <b-button
              type="primary"
              size="large"
              class="action-button"
              @click="reset_device_info"
              icon="ri-refresh-line"
            >
              重置机器码
            </b-button>
          </b-grid-item>

          <b-grid-item column="8">
            <b-button
              type="warning"
              size="large"
              class="action-button"
              @click="restart_cursor"
              :loading="isRestarting"
              icon="ri-restart-line"
            >
              重启 Cursor
            </b-button>
          </b-grid-item>

          <b-grid-item column="8">
            <b-button
              disabled
              type="success"
              size="large"
              class="action-button"
              icon="ri-user-settings-line"
            >
              切换账号 (待开发)
            </b-button>
          </b-grid-item>
        </b-grid>
        <b-divider></b-divider>
        <p class="footer-desc">© 2025 Cursor 重置工具 | 版本 2.0</p>
      </b-footer>
    </b-container>
  </main>
</template>

<style scoped lang="scss">
.container {
  width: 100%;
  height: 100vh;
  box-sizing: border-box;
  padding: 20px;
  display: flex;
  flex-direction: column;
  /* background-color: #f5f7fa; */
  /* 禁止选中 */
  user-select: none;
  overflow-y: auto;
}
.footer-actions {
  .be-grid-item {
    width: 100%;
    .be-button {
      width: 100%;
    }
  }
}
.info-label {
  font-size: 14px;
  color: #4e4848;
  line-height: 1.8rem;
}
.footer-desc {
  text-align: center;
  font-size: 14px;
  color: #9c9a9a;
}
.main-content {
  height: 100%;
  padding: 10px 0px;
  box-sizing: border-box;
  :deep(.be-card .be-card__header) {
    background-color: rgb(241, 241, 241) !important;
    line-height: 30px !important;
  }
  :deep(.be-grid-item) {
    width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  :deep(.be-tag) {
    float: right;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}
</style>
