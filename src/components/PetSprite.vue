<script setup lang="ts">
import { computed } from "vue";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { PetState } from "../types";

const props = defineProps<{
  state: PetState;
  intensity: number;
  paused: boolean;
}>();

const workDuration = computed(() => `${(0.9 - props.intensity * 0.45).toFixed(2)}s`);
const pawDuration = computed(() => `${(0.63 - props.intensity * 0.315).toFixed(2)}s`);

async function onPointerDown(event: PointerEvent) {
  if (event.button !== 0) return;
  event.preventDefault();
  try {
    await getCurrentWindow().startDragging();
  } catch {
    // 窗口不可拖动时忽略，不影响动画状态
  }
}
</script>

<template>
  <div
    class="pet-wrap"
    :class="[state, { paused }]"
    :style="{ '--work-duration': workDuration, '--paw-duration': pawDuration }"
    @pointerdown="onPointerDown"
    @contextmenu.prevent
  >
    <svg class="pet" viewBox="0 0 240 240" role="img" aria-label="MoyuyuPet">
      <path class="tail" d="M178 178 C208 176 218 152 204 138" />

      <path class="ear" d="M66 108 L56 58 L102 86 Z" />
      <path class="ear" d="M174 108 L184 58 L138 86 Z" />
      <path class="ear-inner" d="M72 98 L66 70 L92 86 Z" />
      <path class="ear-inner" d="M168 98 L174 70 L148 86 Z" />

      <ellipse class="body" cx="120" cy="150" rx="72" ry="64" />

      <g class="face">
        <g class="eyes">
          <ellipse class="eye" cx="94" cy="140" rx="6.5" ry="9" />
          <ellipse class="eye" cx="146" cy="140" rx="6.5" ry="9" />
        </g>
        <g class="closed-eyes">
          <path d="M84 140 Q94 149 104 140" />
          <path d="M136 140 Q146 149 156 140" />
        </g>
        <ellipse class="cheek" cx="76" cy="158" rx="11" ry="6" />
        <ellipse class="cheek" cx="164" cy="158" rx="11" ry="6" />
        <path class="mouth" d="M112 156 Q120 165 128 156" />
      </g>

      <g class="paw paw-left">
        <ellipse cx="86" cy="200" rx="19" ry="11" />
      </g>
      <g class="paw paw-right">
        <ellipse cx="154" cy="200" rx="19" ry="11" />
      </g>

      <g class="sparkles">
        <circle cx="46" cy="88" r="4" />
        <circle cx="198" cy="104" r="3" />
        <circle cx="72" cy="54" r="3" />
      </g>

      <path class="sweat" d="M190 142 Q198 154 190 162 Q182 154 190 142 Z" />

      <g class="zzz">
        <text x="184" y="92">Z</text>
        <text x="200" y="74">z</text>
      </g>
    </svg>
  </div>
</template>

<style scoped>
.pet-wrap {
  width: 220px;
  height: 220px;
  cursor: grab;
  user-select: none;
  touch-action: none;
}

.pet-wrap:active {
  cursor: grabbing;
}

.pet {
  width: 100%;
  height: 100%;
  overflow: visible;
  transform-box: fill-box;
  transform-origin: 50% 100%;
}

.body,
.ear {
  fill: #f9dfc0;
  stroke: #5b3a29;
  stroke-width: 4;
}

.ear {
  stroke-linejoin: round;
}

.ear-inner {
  fill: #f3b7b4;
}

.eye {
  fill: #4a2f21;
}

.closed-eyes {
  display: none;
  fill: none;
  stroke: #4a2f21;
  stroke-width: 4;
  stroke-linecap: round;
}

.cheek {
  fill: #f3a6a0;
  opacity: 0.55;
}

.mouth {
  fill: none;
  stroke: #4a2f21;
  stroke-width: 3;
  stroke-linecap: round;
}

.paw {
  fill: #f4cfae;
  stroke: #5b3a29;
  stroke-width: 3;
}

.tail {
  fill: none;
  stroke: #5b3a29;
  stroke-width: 10;
  stroke-linecap: round;
  transform-box: fill-box;
  transform-origin: 0% 90%;
  animation: wag 2.6s ease-in-out infinite;
}

.sparkles {
  opacity: 0;
  fill: #f6c453;
  transform-box: fill-box;
  transform-origin: center;
}

.sweat {
  opacity: 0;
  fill: #8fc7e8;
  stroke: #5b3a29;
  stroke-width: 2;
  transform-box: fill-box;
  transform-origin: center;
}

.zzz {
  opacity: 0;
  fill: #6d5a4d;
}

.zzz text {
  font-size: 18px;
  font-weight: 700;
  transform-box: fill-box;
  transform-origin: center;
}

.idle .pet {
  animation: breathe 3.4s ease-in-out infinite;
}

.working .pet {
  animation: bounce var(--work-duration) ease-in-out infinite;
}

.working .paw-left {
  animation: type-left var(--paw-duration) ease-in-out infinite;
}

.working .paw-right {
  animation: type-right var(--paw-duration) ease-in-out infinite;
}

.working .tail {
  animation-duration: var(--work-duration);
}

.celebrate .pet {
  animation: jump 0.9s ease-in-out infinite;
}

.celebrate .sparkles {
  animation: sparkle 0.9s ease-in-out infinite;
}

.error .pet {
  animation: shake 0.35s ease-in-out infinite;
}

.error .sweat {
  animation: sweat 1.1s ease-in-out infinite;
}

.sleep .pet {
  animation: breathe 4.8s ease-in-out infinite;
}

.sleep .eyes {
  display: none;
}

.sleep .closed-eyes {
  display: block;
}

.sleep .zzz {
  opacity: 1;
}

.sleep .zzz text {
  animation: zzz 2.4s ease-in-out infinite;
}

.sleep .zzz text:nth-child(2) {
  animation-delay: 0.8s;
}

.paused .pet,
.paused .paw,
.paused .tail,
.paused .sparkles,
.paused .sweat,
.paused .zzz text {
  animation-play-state: paused;
}

@keyframes breathe {
  0%,
  100% {
    transform: scale(1) translateY(0);
  }
  50% {
    transform: scale(1.025, 0.985) translateY(1px);
  }
}

@keyframes bounce {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-6px);
  }
}

@keyframes type-left {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-12px);
  }
}

@keyframes type-right {
  0%,
  100% {
    transform: translateY(-12px);
  }
  50% {
    transform: translateY(0);
  }
}

@keyframes jump {
  0%,
  100% {
    transform: translateY(0) scale(1);
  }
  35% {
    transform: translateY(-26px) scale(1.04);
  }
  65% {
    transform: translateY(-5px) scale(0.99);
  }
}

@keyframes shake {
  0%,
  100% {
    transform: translateX(0) rotate(0);
  }
  20% {
    transform: translateX(-6px) rotate(-2deg);
  }
  40% {
    transform: translateX(6px) rotate(2deg);
  }
  60% {
    transform: translateX(-4px) rotate(-1deg);
  }
  80% {
    transform: translateX(4px) rotate(1deg);
  }
}

@keyframes wag {
  0%,
  100% {
    transform: rotate(0);
  }
  50% {
    transform: rotate(10deg);
  }
}

@keyframes sparkle {
  0%,
  100% {
    opacity: 0;
    transform: scale(0.6);
  }
  50% {
    opacity: 1;
    transform: scale(1.1);
  }
}

@keyframes sweat {
  0%,
  100% {
    opacity: 0;
    transform: translateY(0);
  }
  40% {
    opacity: 1;
  }
  80% {
    opacity: 0;
    transform: translateY(10px);
  }
}

@keyframes zzz {
  0% {
    opacity: 0;
    transform: translate(0, 0) scale(0.8);
  }
  30% {
    opacity: 1;
  }
  100% {
    opacity: 0;
    transform: translate(8px, -16px) scale(1.1);
  }
}

@media (prefers-reduced-motion: reduce) {
  .pet,
  .paw,
  .tail,
  .sparkles,
  .sweat,
  .zzz text {
    animation: none;
  }
}
</style>
