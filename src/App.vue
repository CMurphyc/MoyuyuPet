<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import PetStage from "./components/PetStage.vue";
import SettingsView from "./components/SettingsView.vue";

const isSettings = getCurrentWindow().label === "settings";
document.body.dataset.window = isSettings ? "settings" : "pet";
</script>

<template>
  <SettingsView v-if="isSettings" />
  <PetStage v-else />
</template>

<style>
html,
body,
#app {
  height: 100%;
  margin: 0;
}

body {
  font-family: "Microsoft YaHei", "Segoe UI", sans-serif;
}

body[data-window="pet"] {
  overflow: hidden;
  background: transparent;
}

body[data-window="settings"] {
  overflow-y: auto;
  background: #f4f1ec;
  color: #3f2f25;
}
</style>
