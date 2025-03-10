<script setup lang="ts">
import { onMounted, ref } from "vue";
// import { invoke } from "@tauri-apps/api/core";
import Switch from './components/Switch.vue';

import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
import { register, isRegistered, unregister } from '@tauri-apps/plugin-deep-link';
import { getVersion, getName } from "@tauri-apps/api/app";
import { getCurrentWindow } from '@tauri-apps/api/window';

import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

const autoStartValue = ref(false);
const customProtocolValue = ref(false);

// 通知启动
async function notificationStartd() {
  // Do you have permission to send a notification?
  let permissionGranted = await isPermissionGranted();

  // If not we need to request it
  if (!permissionGranted) {
    const permission = await requestPermission();
    permissionGranted = permission === 'granted';
  }

  // Once permission has been granted we can send the notification
  if (permissionGranted) {
    sendNotification({ title: 'Opener 已经启动', body: '启动后将自动进入后台运行' });
  }
}

async function updateVerionToTitle() {
  let cw = getCurrentWindow();
  let v = await getVersion();
  await cw.setTitle(await getName() + " for v" + v);
}


onMounted(async () => {
  autoStartValue.value = await isEnabled();
  console.log(`enable for autostart? ${autoStartValue.value}`);

  try {
    customProtocolValue.value = await isRegistered("opener");
  } catch (e) {
    console.error(e);
  }


  await notificationStartd()
  await updateVerionToTitle()

  console.log(`registered for custom protocol? ${customProtocolValue.value}`);
})

// 开机启动控制
async function toggleAutoStart(checked: boolean) {
  if (checked) {
    console.log("enable autostart");
    await enable();
    sendNotification({ title: '开机启动', body: '开机启动启用后，将加快打开速度', group: 'action' });
  } else {
    console.log("disable autostart");
    await disable();
  }
  // autoStartValue.value = checked;
}

async function toggleCustomProtocol(checked: boolean) {
  if (checked) {
    console.log("register custom protocol");
    await register("opener");
  } else {
    console.log("unregister custom protocol");
    await unregister("opener");
    sendNotification({ title: '移除协议注册', body: '移除协议注册后，将不能打开本地文件和路径', group: 'action' });
  }
}
</script>

<template>
  <main class="container mx-auto p-4">
    <h1 class="text-2xl font-bold mb-4">Welcome to Opener</h1>

    <div class="flex justify-center space-x-4">
      <div class="switch-item bg-white shadow-md rounded-lg p-4 w-64">
        <Switch v-model="autoStartValue" @update:modelValue="toggleAutoStart" />
        <p class="mt-2">开机启动 is {{ autoStartValue ? 'On' : 'Off' }}</p>
      </div>

      <div class="switch-item bg-white shadow-md rounded-lg p-4 w-64">
        <Switch v-model="customProtocolValue" @update:modelValue="toggleCustomProtocol" />
        <p class="mt-2">自定义协议 is {{ customProtocolValue ? 'Registered' : 'No' }}</p>
      </div>
    </div>
  </main>
</template>

<style scoped>
/* 如果需要额外的样式，可以在这里添加 */
</style>
<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 10vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

h1 {
  text-align: center;
}
</style>