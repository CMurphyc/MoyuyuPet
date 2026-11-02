<script setup lang="ts">
import PetSprite from "./PetSprite.vue";
import SpeechBubble from "./SpeechBubble.vue";
import { usePetActivity } from "../composables/usePetActivity";
import { useSpeechBubble } from "../composables/useSpeechBubble";

const { state, intensity, paused } = usePetActivity();
const { message } = useSpeechBubble(() => paused.value);
</script>

<template>
  <main class="stage">
    <div class="bubble-slot">
      <SpeechBubble :message="message" />
    </div>
    <PetSprite :state="state" :intensity="intensity" :paused="paused" />
  </main>
</template>

<style scoped>
.stage {
  box-sizing: border-box;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-end;
  width: 100%;
  height: 100%;
  padding-bottom: 8px;
  user-select: none;
}

.bubble-slot {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  width: 100%;
  height: 150px;
  margin-bottom: 2px;
}
</style>
