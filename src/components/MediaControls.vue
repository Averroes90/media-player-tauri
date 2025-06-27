<template>
  <v-card variant="outlined" class="mt-4">
    <v-card-title>
      <v-icon left>mdi-play-circle</v-icon>
      Media Controls
    </v-card-title>

    <v-card-text>
      <!-- Basic Playback Controls -->
      <v-row class="mb-4">
        <v-col>
          <v-btn
            @click="togglePlayPause"
            :color="isPlaying ? 'warning' : 'success'"
            :disabled="!isInitialized"
            block
            prepend-icon="mdi-play-pause"
          >
            {{ isPlaying ? "Pause" : "Play" }}
          </v-btn>
        </v-col>

        <v-col>
          <v-btn
            @click="stopVideo"
            color="error"
            :disabled="!isInitialized"
            block
            prepend-icon="mdi-stop"
          >
            Stop
          </v-btn>
        </v-col>
      </v-row>

      <!-- Speed Controls Section -->
      <v-divider class="mb-4"></v-divider>

      <v-row align="center" class="mb-3">
        <v-col cols="auto">
          <h4>Playback Speed</h4>
        </v-col>
        <v-col cols="auto">
          <v-chip
            :color="currentSpeed === 1.0 ? 'primary' : 'default'"
            size="small"
          >
            {{ currentSpeed }}x
          </v-chip>
        </v-col>
      </v-row>

      <!-- Speed Preset Buttons -->
      <v-row class="mb-3">
        <v-col v-for="speed in speedPresets" :key="speed" cols="auto">
          <v-btn
            @click="setSpeedPreset(speed)"
            :color="currentSpeed === speed ? 'primary' : 'default'"
            :variant="currentSpeed === speed ? 'flat' : 'outlined'"
            :disabled="!isInitialized"
            size="small"
          >
            {{ speed }}x
          </v-btn>
        </v-col>
      </v-row>

      <!-- Custom Speed Input -->
      <v-row>
        <v-col cols="8">
          <v-text-field
            v-model.number="customSpeed"
            label="Custom Speed"
            type="number"
            step="0.1"
            min="0.1"
            max="4.0"
            :disabled="!isInitialized"
            density="compact"
            @keyup.enter="setCustomSpeed"
          />
        </v-col>
        <v-col cols="4">
          <v-btn
            @click="setCustomSpeed"
            color="secondary"
            :disabled="!isInitialized || !customSpeed"
            block
          >
            Set
          </v-btn>
        </v-col>
      </v-row>

      <!-- Quick Speed Adjustments -->
      <v-row class="mt-2">
        <v-col>
          <v-btn
            @click="adjustSpeed(-0.25)"
            color="blue-grey"
            :disabled="!isInitialized"
            block
            size="small"
            prepend-icon="mdi-minus"
          >
            -0.25x
          </v-btn>
        </v-col>
        <v-col>
          <v-btn
            @click="resetSpeed"
            color="blue-grey"
            :disabled="!isInitialized"
            block
            size="small"
            prepend-icon="mdi-restore"
          >
            Reset (1x)
          </v-btn>
        </v-col>
        <v-col>
          <v-btn
            @click="adjustSpeed(0.25)"
            color="blue-grey"
            :disabled="!isInitialized"
            block
            size="small"
            prepend-icon="mdi-plus"
          >
            +0.25x
          </v-btn>
        </v-col>
      </v-row>
    </v-card-text>
  </v-card>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

// Props
const props = defineProps({
  isInitialized: {
    type: Boolean,
    required: true,
  },
});

// Emits
const emit = defineEmits(["status-update"]);

// Reactive state
const isPlaying = ref(false);
const currentSpeed = ref(1.0);
const customSpeed = ref(1.0);

// Speed presets
const speedPresets = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0];

// Helper to emit status updates
const emitStatus = (type, message) => {
  emit("status-update", type, message);
};

// Basic playback controls
const togglePlayPause = async () => {
  try {
    const result = await invoke("play_pause");
    isPlaying.value = !isPlaying.value;
    emitStatus("info", result);
  } catch (error) {
    emitStatus("error", `Failed to toggle playback: ${error}`);
  }
};

const stopVideo = async () => {
  try {
    const result = await invoke("stop_video");
    isPlaying.value = false;
    emitStatus("info", result);
  } catch (error) {
    emitStatus("error", `Failed to stop video: ${error}`);
  }
};

// Speed control functions
const setSpeedPreset = async (speed) => {
  try {
    const result = await invoke("speed_preset", { speed });
    currentSpeed.value = speed;
    customSpeed.value = speed;
    emitStatus("success", `Speed set to ${speed}x`);
  } catch (error) {
    emitStatus("error", `Failed to set speed: ${error}`);
  }
};

const setCustomSpeed = async () => {
  if (
    !customSpeed.value ||
    customSpeed.value < 0.1 ||
    customSpeed.value > 4.0
  ) {
    emitStatus("warning", "Speed must be between 0.1x and 4.0x");
    return;
  }

  try {
    const result = await invoke("set_playback_speed", {
      speed: customSpeed.value,
    });
    currentSpeed.value = customSpeed.value;
    emitStatus("success", `Speed set to ${customSpeed.value}x`);
  } catch (error) {
    emitStatus("error", `Failed to set speed: ${error}`);
  }
};

const adjustSpeed = async (adjustment) => {
  const newSpeed = Math.max(
    0.1,
    Math.min(4.0, currentSpeed.value + adjustment)
  );
  customSpeed.value = newSpeed;
  await setCustomSpeed();
};

const resetSpeed = async () => {
  customSpeed.value = 1.0;
  await setCustomSpeed();
};

const getCurrentSpeed = async () => {
  if (!props.isInitialized) return;

  try {
    const speed = await invoke("get_playback_speed");
    currentSpeed.value = speed;
    customSpeed.value = speed;
  } catch (error) {
    emitStatus("error", `Failed to get current speed: ${error}`);
  }
};

// Get current speed when component mounts and when initialized
onMounted(() => {
  if (props.isInitialized) {
    getCurrentSpeed();
  }
});

// Watch for initialization changes
import { watch } from "vue";
watch(
  () => props.isInitialized,
  (newVal) => {
    if (newVal) {
      getCurrentSpeed();
    }
  }
);
</script>

<style scoped>
.v-btn {
  text-transform: none;
}
</style>
