import { onMounted, onUnmounted, ref } from "vue";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import templates from "../assets/bubbles/zh-CN.json";
import type { BubbleKind, BubblePayload } from "../types";

const COOLDOWN_MS = 40000;
const SHOW_MS = 6000;
const MAX_LENGTH = 30;
const MAX_QUEUE = 5;

interface QueueItem {
  text: string;
  priority: number;
}

const priority: Record<BubbleKind, number> = {
  error: 5,
  celebrate: 4,
  working: 2,
  idle: 1,
};

const catalogue = templates as Record<string, string[]>;

export function useSpeechBubble(isPaused: () => boolean) {
  const message = ref<string | null>(null);
  const queue: QueueItem[] = [];
  let lastShownAt = 0;
  let showTimer: number | undefined;
  let pumpTimer: number | undefined;
  const unlisteners: UnlistenFn[] = [];

  function render(template: string, vars: Record<string, string | number>) {
    return template.replace(/\{(\w+)\}/g, (_, key: string) => String(vars[key] ?? ""));
  }

  function pump() {
    if (message.value || isPaused() || queue.length === 0) return;

    queue.sort((a, b) => b.priority - a.priority);
    const next = queue[0];
    const now = Date.now();
    if (now - lastShownAt < COOLDOWN_MS && next.priority < 4) return;

    queue.shift();
    message.value = next.text;
    lastShownAt = now;
    showTimer = window.setTimeout(() => {
      message.value = null;
      showTimer = undefined;
    }, SHOW_MS);
  }

  function enqueue(kind: BubbleKind, vars: Record<string, string | number> = {}) {
    if (isPaused()) return;

    const list = catalogue[kind];
    if (!list?.length) return;

    const raw = list[Math.floor(Math.random() * list.length)];
    const text = render(raw, vars).slice(0, MAX_LENGTH);
    queue.push({ text, priority: priority[kind] });

    if (queue.length > MAX_QUEUE) {
      queue.sort((a, b) => b.priority - a.priority);
      queue.length = MAX_QUEUE;
    }

    pump();
  }

  onMounted(async () => {
    unlisteners.push(
      await listen<BubblePayload>("pet://bubble", (event) => {
        enqueue(event.payload.kind, {
          tool: event.payload.tool ?? "AI",
          tokens: event.payload.tokens ?? 0,
        });
      }),
    );
    unlisteners.push(
      await listen<boolean>("pet://paused", (event) => {
        if (!event.payload) return;
        queue.length = 0;
        message.value = null;
        if (showTimer !== undefined) {
          window.clearTimeout(showTimer);
          showTimer = undefined;
        }
      }),
    );

    pumpTimer = window.setInterval(pump, 1000);
  });

  onUnmounted(() => {
    if (showTimer !== undefined) window.clearTimeout(showTimer);
    if (pumpTimer !== undefined) window.clearInterval(pumpTimer);
    for (const unlisten of unlisteners) unlisten();
  });

  return { message };
}
