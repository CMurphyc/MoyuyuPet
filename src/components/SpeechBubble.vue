<script setup lang="ts">
defineProps<{
  message: string | null;
}>();
</script>

<template>
  <Transition name="bubble">
    <div v-if="message" class="bubble" role="status" aria-live="polite">
      <span class="text">{{ message }}</span>
      <span class="tail" aria-hidden="true"></span>
    </div>
  </Transition>
</template>

<style scoped>
.bubble {
  position: relative;
  max-width: 300px;
  min-height: 44px;
  padding: 11px 16px;
  border-radius: 18px;
  border: 2px solid #5b3a29;
  background: rgba(255, 252, 245, 0.96);
  box-shadow: 0 8px 24px rgba(70, 40, 20, 0.18);
  color: #4a2f21;
  font-size: 15px;
  line-height: 1.5;
  text-align: center;
  word-break: break-word;
}

.text {
  display: block;
}

.tail {
  position: absolute;
  left: 50%;
  bottom: -9px;
  width: 16px;
  height: 16px;
  border-right: 2px solid #5b3a29;
  border-bottom: 2px solid #5b3a29;
  border-bottom-right-radius: 4px;
  background: rgba(255, 252, 245, 0.96);
  transform: translateX(-50%) rotate(45deg);
}

.bubble-enter-active,
.bubble-leave-active {
  transition:
    opacity 0.18s ease,
    transform 0.18s ease;
}

.bubble-enter-from,
.bubble-leave-to {
  opacity: 0;
  transform: translateY(6px) scale(0.96);
}
</style>
