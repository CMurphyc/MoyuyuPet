<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { Application } from "pixi.js";
import type { Live2DModel } from "pixi-live2d-display/cubism4";
import type { PetState } from "../types";

declare global {
  interface Window {
    Live2DCubismCore?: unknown;
  }
}

const props = defineProps<{
  state: PetState;
  intensity: number;
  paused: boolean;
}>();

const emit = defineEmits<{
  unavailable: [];
}>();

const host = ref<HTMLDivElement | null>(null);
let app: Application | undefined;
let model: Live2DModel | undefined;
let workTimer: number | undefined;
let sleeping = false;
let disposed = false;

const VIEW_WIDTH = 320;
const VIEW_HEIGHT = 300;

async function loadScript(url: string) {
  if (window.Live2DCubismCore) return;

  await new Promise<void>((resolve, reject) => {
    const script = document.createElement("script");
    script.src = url;
    script.async = true;
    script.onload = () => resolve();
    script.onerror = () => {
      script.remove();
      reject(new Error(`failed to load ${url}`));
    };
    document.head.appendChild(script);
  });
}

function clearWorkTimer() {
  if (workTimer !== undefined) {
    window.clearInterval(workTimer);
    workTimer = undefined;
  }
}

function play(group: string) {
  if (!model) return;
  void model.motion(group).catch(() => {
    // Some models don't provide every motion group; keep the pet idle instead.
  });
}

function applyState() {
  if (!model || !app) return;

  sleeping = props.state === "sleep";
  clearWorkTimer();

  if (props.paused) {
    app.ticker.stop();
    return;
  }

  app.ticker.start();

  switch (props.state) {
    case "working":
      play("Tap");
      workTimer = window.setInterval(
        () => play("Tap"),
        Math.max(900, 2200 - props.intensity * 1200),
      );
      break;
    case "celebrate":
      play("Flick");
      break;
    case "error":
      play("FlickDown");
      break;
    case "sleep":
      model.internalModel.motionManager.stopAllMotions();
      break;
    default:
      play("Idle");
      break;
  }
}

async function initialize() {
  if (!host.value) return;

  try {
    const modelUrl = new URL("live2d/tororo/tororo.model3.json", window.location.href).href;
    const response = await fetch(modelUrl, { cache: "no-store" });
    if (!response.ok) throw new Error("Live2D model not installed");

    await loadScript(new URL("live2d/core/live2dcubismcore.min.js", window.location.href).href);

    const PIXI = await import("pixi.js");
    const { Live2DModel: Live2DModelClass } = await import(
      "pixi-live2d-display/cubism4"
    );

    Live2DModelClass.registerTicker(PIXI.Ticker);

    const canvas = document.createElement("canvas");
    canvas.className = "live2d-canvas";
    host.value.appendChild(canvas);

    app = new PIXI.Application({
      view: canvas,
      width: VIEW_WIDTH,
      height: VIEW_HEIGHT,
      backgroundAlpha: 0,
      antialias: true,
      autoStart: true,
      resolution: Math.min(window.devicePixelRatio || 1, 2),
      autoDensity: true,
    });

    const loaded = await Live2DModelClass.from(modelUrl, { autoInteract: false });
    if (disposed) {
      loaded.destroy();
      return;
    }

    model = loaded;
    app.stage.addChild(model);

    const modelWidth = model.internalModel.width;
    const modelHeight = model.internalModel.height;
    const scale = Math.min((VIEW_WIDTH * 0.92) / modelWidth, (VIEW_HEIGHT * 0.98) / modelHeight);

    model.anchor.set(0.5, 1);
    model.scale.set(scale);
    model.position.set(VIEW_WIDTH / 2, VIEW_HEIGHT - 2);
    model.interactive = false;

    model.internalModel.on("beforeModelUpdate", () => {
      if (!sleeping || !model) return;
      const core = model.internalModel.coreModel as unknown as {
        setParameterValueById(id: string, value: number): void;
      };
      core.setParameterValueById("PARAM_EYE_L_OPEN", 0);
      core.setParameterValueById("PARAM_EYE_R_OPEN", 0);
      core.setParameterValueById("PARAM_MOUTH_OPEN_Y", 0);
    });

    applyState();
  } catch (error) {
    console.warn("[live2d] unavailable, falling back to SVG pet", error);
    if (!disposed) emit("unavailable");
  }
}

async function onPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  event.preventDefault();
  try {
    await getCurrentWindow().startDragging();
  } catch {
    // Ignore drag failures when the window is not interactive.
  }
}

watch(
  () => [props.state, props.intensity, props.paused] as const,
  () => applyState(),
);

onMounted(() => {
  void initialize();
});

onUnmounted(() => {
  disposed = true;
  clearWorkTimer();
  model?.internalModel.off("beforeModelUpdate");
  model?.destroy();
  app?.destroy(true, { children: true });
  model = undefined;
  app = undefined;
});
</script>

<template>
  <div
    class="live2d-wrap"
    @pointerdown="onPointerDown"
    @contextmenu.prevent
  >
    <div ref="host" class="live2d-host"></div>
  </div>
</template>

<style scoped>
.live2d-wrap {
  width: 320px;
  height: 300px;
  cursor: grab;
  user-select: none;
  touch-action: none;
}

.live2d-wrap:active {
  cursor: grabbing;
}

.live2d-host {
  width: 100%;
  height: 100%;
}

.live2d-host :deep(canvas) {
  display: block;
  width: 100%;
  height: 100%;
  pointer-events: none;
}
</style>
