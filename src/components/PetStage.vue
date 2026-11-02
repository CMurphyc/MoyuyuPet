<script setup lang="ts">
import { ref } from "vue";
import Live2DPet from "./Live2DPet.vue";
import PetSprite from "./PetSprite.vue";
import SpeechBubble from "./SpeechBubble.vue";
import { usePetActivity } from "../composables/usePetActivity";
import { useSpeechBubble } from "../composables/useSpeechBubble";

const { state, intensity, paused } = usePetActivity();
const { message } = useSpeechBubble(() => paused.value);
const useLive2D = ref(true);
</script>

<template>
  <main class="stage">
    <div class="bubble-slot">
      <SpeechBubble :message="message" />
    </div>
    <Live2DPet
      v-if="useLive2D"
      :state="state"
      :intensity="intensity"
      :paused="paused"
      @unavailable="useLive2D = false"
    />
    <PetSprite v-else :state="state" :intensity="intensity" :paused="paused" />
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
  height: 120px;
  margin-bottom: 2px;
}
</style>
