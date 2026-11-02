import { onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ActivityPayload, PetState } from "../types";

const IDLE_AFTER_MS = 3000;
const SLEEP_AFTER_MS = 30000;

const priority: Record<PetState, number> = {
  error: 4,
  celebrate: 3,
  working: 2,
  idle: 1,
  sleep: 0,
};

export function usePetActivity() {
  const state = ref<PetState>("idle");
  const intensity = ref(0);
  const tokenRate = ref(0);
  const paused = ref(false);

  let idleTimer: number | undefined;
  let sleepTimer: number | undefined;
  let intensityTimer: number | undefined;
  let targetIntensity = 0;
  let lockedState: PetState | null = null;
  let lockedUntil = 0;
  const unlisteners: UnlistenFn[] = [];

  function clearTimer(timer: number | undefined) {
    if (timer !== undefined) window.clearTimeout(timer);
  }

  function clearTimers() {
    clearTimer(idleTimer);
    clearTimer(sleepTimer);
    idleTimer = undefined;
    sleepTimer = undefined;
  }

  function scheduleSleep() {
    clearTimer(sleepTimer);
    sleepTimer = window.setTimeout(() => {
      if (state.value === "idle") state.value = "sleep";
    }, SLEEP_AFTER_MS);
  }

  function scheduleIdle() {
    clearTimer(idleTimer);
    idleTimer = window.setTimeout(() => {
      state.value = "idle";
      targetIntensity = 0;
      tokenRate.value = 0;
      lockedState = null;
      scheduleSleep();
    }, IDLE_AFTER_MS);
  }

  function setTargetIntensity(value: number) {
    targetIntensity = Math.min(1, Math.max(0, value));
  }

  function apply(payload: ActivityPayload) {
    if (paused.value) return;

    const now = Date.now();
    if (lockedState && now < lockedUntil && priority[payload.state] <= priority[lockedState]) {
      return;
    }

    lockedState = null;
    state.value = payload.state;
    setTargetIntensity(payload.intensity);
    tokenRate.value = payload.token_rate;

    clearTimers();
    switch (payload.state) {
      case "working":
        scheduleIdle();
        break;
      case "celebrate":
      case "error":
        lockedState = payload.state;
        lockedUntil = now + IDLE_AFTER_MS;
        scheduleIdle();
        break;
      case "idle":
        setTargetIntensity(0);
        scheduleSleep();
        break;
      case "sleep":
        setTargetIntensity(0);
        break;
    }
  }

  onMounted(async () => {
    unlisteners.push(
      await listen<ActivityPayload>("pet://activity", (event) => apply(event.payload)),
    );
    unlisteners.push(
      await listen<boolean>("pet://paused", (event) => {
        paused.value = event.payload;
        if (event.payload) {
          clearTimers();
        } else {
          state.value = "idle";
          targetIntensity = 0;
          intensity.value = 0;
          scheduleSleep();
        }
      }),
    );
    intensityTimer = window.setInterval(() => {
      const diff = targetIntensity - intensity.value;
      intensity.value =
        Math.abs(diff) < 0.01 ? targetIntensity : intensity.value + diff * 0.2;
    }, 100);
    scheduleSleep();
  });

  onUnmounted(() => {
    clearTimers();
    clearTimer(intensityTimer);
    for (const unlisten of unlisteners) unlisten();
  });

  return { state, intensity, tokenRate, paused };
}
