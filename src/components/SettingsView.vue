<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { AdapterStatus, AppSettings } from "../types";

const settings = ref<AppSettings>({ paused: false, autostart: false, include_task: true });
const statuses = ref<AdapterStatus[]>([]);
const error = ref<string | null>(null);
const loading = ref(true);
const unlisteners: UnlistenFn[] = [];

const sourceLabels: Record<string, string> = {
  qoder: "Qoder / Qoder CN",
  dsh: "DeepSeek Harness",
};

const statusLabels: Record<string, string> = {
  connected: "已连接",
  not_found: "未检测到",
  busy: "数据库忙",
  error: "解析失败",
};

function statusLabel(status: string) {
  return statusLabels[status] ?? status;
}

function upsertStatus(status: AdapterStatus) {
  const index = statuses.value.findIndex((item) => item.source === status.source);
  if (index >= 0) {
    statuses.value[index] = status;
  } else {
    statuses.value.push(status);
  }
}

async function load() {
  loading.value = true;
  error.value = null;
  try {
    settings.value = await invoke<AppSettings>("get_settings");
    statuses.value = await invoke<AdapterStatus[]>("get_adapter_statuses");
  } catch (cause) {
    error.value = String(cause);
  } finally {
    loading.value = false;
  }
}

async function togglePause() {
  const paused = !settings.value.paused;
  try {
    await invoke("set_paused", { paused });
    settings.value.paused = paused;
  } catch (cause) {
    error.value = String(cause);
  }
}

async function toggleAutostart() {
  const enabled = !settings.value.autostart;
  try {
    settings.value.autostart = await invoke<boolean>("set_autostart", { enabled });
  } catch (cause) {
    error.value = String(cause);
  }
}

async function toggleIncludeTask() {
  const includeTask = !settings.value.include_task;
  try {
    settings.value.include_task = await invoke<boolean>("set_include_task", {
      includeTask,
    });
  } catch (cause) {
    error.value = String(cause);
  }
}

onMounted(async () => {
  await load();
  unlisteners.push(
    await listen<AdapterStatus>("pet://adapter-status", (event) => {
      upsertStatus(event.payload);
    }),
  );
  unlisteners.push(
    await listen<boolean>("pet://paused", (event) => {
      settings.value.paused = event.payload;
    }),
  );
});

onUnmounted(() => {
  for (const unlisten of unlisteners) unlisten();
});
</script>

<template>
  <main class="settings">
    <header class="hero">
      <div>
        <h1>MoyuyuPet</h1>
        <p>本地优先的 AI 桌宠 · 只在你需要时出现</p>
      </div>
      <span class="version">v0.1.0</span>
    </header>

    <section class="card">
      <h2>运行</h2>
      <button class="row" :aria-pressed="settings.paused" @click="togglePause">
        <span class="row-text">
          <strong>暂停宠物</strong>
          <small>暂停动画、气泡与适配器事件</small>
        </span>
        <span class="switch" :class="{ on: settings.paused }"><span></span></span>
      </button>
      <button class="row" :aria-pressed="settings.autostart" @click="toggleAutostart">
        <span class="row-text">
          <strong>开机自启</strong>
          <small>登录 Windows 后自动启动</small>
        </span>
        <span class="switch" :class="{ on: settings.autostart }"><span></span></span>
      </button>
      <button class="row" :aria-pressed="settings.include_task" @click="toggleIncludeTask">
        <span class="row-text">
          <strong>气泡包含任务标题</strong>
          <small>只截取会话标题，最多 24 字，可随时关闭</small>
        </span>
        <span class="switch" :class="{ on: settings.include_task }"><span></span></span>
      </button>
    </section>

    <section class="card">
      <h2>适配器</h2>
      <p v-if="loading" class="muted">正在读取状态…</p>
      <ul v-else class="adapters">
        <li v-for="status in statuses" :key="status.source">
          <span class="dot" :class="status.status"></span>
          <span class="adapter-text">
            <strong>{{ sourceLabels[status.source] ?? status.source }}</strong>
            <small>{{ status.message || statusLabel(status.status) }}</small>
          </span>
          <span class="badge" :class="status.status">{{ statusLabel(status.status) }}</span>
        </li>
        <li v-if="!statuses.length" class="muted">等待适配器上报状态…</li>
      </ul>
    </section>

    <section class="card privacy">
      <h2>隐私</h2>
      <p>
        只读取本地元数据：Qoder 的 token_info 与会话标题、dsh 的 usage、turn 状态与标题。
        任务标题最多保留 24 字，可在上方关闭。不读取对话正文，不联网，不上传。
      </p>
    </section>

    <p v-if="error" class="error">{{ error }}</p>
    <footer class="footer">配置保存在本地 · 关闭窗口不会退出宠物</footer>
  </main>
</template>

<style scoped>
.settings {
  max-width: 520px;
  margin: 0 auto;
  padding: 28px 24px 40px;
}

.hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 18px;
}

h1 {
  margin: 0;
  font-size: 24px;
}

.hero p {
  margin: 6px 0 0;
  color: #7a685c;
  font-size: 13px;
}

.version {
  flex: none;
  padding: 4px 10px;
  border-radius: 999px;
  background: #e9e2d9;
  color: #8a776b;
  font-size: 12px;
}

.card {
  margin-bottom: 14px;
  padding: 16px 18px;
  border: 1px solid #e6ddd3;
  border-radius: 16px;
  background: #fffdf9;
  box-shadow: 0 4px 16px rgba(70, 40, 20, 0.05);
}

.card h2 {
  margin: 0 0 10px;
  font-size: 15px;
}

.row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  width: 100%;
  padding: 10px 0;
  border: 0;
  background: transparent;
  color: inherit;
  font: inherit;
  text-align: left;
  cursor: pointer;
}

.row + .row {
  border-top: 1px dashed #eee3d8;
}

.row-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.row-text strong {
  font-size: 14px;
}

.row-text small {
  color: #8a776b;
  font-size: 12px;
}

.switch {
  box-sizing: border-box;
  flex: none;
  width: 44px;
  height: 26px;
  padding: 3px;
  border-radius: 999px;
  background: #ddd3c8;
  transition: background 0.18s ease;
}

.switch span {
  display: block;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  transition: transform 0.18s ease;
}

.switch.on {
  background: #e08b5b;
}

.switch.on span {
  transform: translateX(18px);
}

.adapters {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.adapters li {
  display: flex;
  align-items: center;
  gap: 10px;
}

.dot {
  flex: none;
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #b8aca1;
}

.dot.connected {
  background: #5aab7a;
}

.dot.busy {
  background: #e0a33b;
}

.dot.error {
  background: #d96a5f;
}

.adapter-text {
  display: flex;
  flex: 1;
  flex-direction: column;
  gap: 2px;
}

.adapter-text strong {
  font-size: 13px;
}

.adapter-text small {
  color: #8a776b;
  font-size: 11px;
}

.badge {
  padding: 3px 8px;
  border-radius: 999px;
  background: #f1eae2;
  color: #7a685c;
  font-size: 11px;
}

.badge.connected {
  background: #e5f3e9;
  color: #2f7a4f;
}

.badge.busy {
  background: #fbf0d9;
  color: #93651d;
}

.badge.error {
  background: #fbe6e3;
  color: #a33d33;
}

.privacy p {
  margin: 0;
  color: #6e5d51;
  font-size: 13px;
  line-height: 1.7;
}

.muted {
  color: #9a887b;
  font-size: 12px;
}

.error {
  color: #a33d33;
  font-size: 12px;
}

.footer {
  margin-top: 10px;
  text-align: center;
  color: #a08e80;
  font-size: 11px;
}
</style>
