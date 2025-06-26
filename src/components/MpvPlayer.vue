<template>
  <v-container>
    <v-row>
      <v-col cols="12">
        <v-card>
          <v-card-title>MPV Media Player</v-card-title>

          <v-card-text>
            <!-- Status display -->
            <v-alert
              :type="status.type"
              :text="status.message"
              v-if="status.message"
              class="mb-4"
            />

            <!-- File selection -->
            <v-text-field
              v-model="videoPath"
              label="Video file path"
              placeholder="/path/to/your/video.mp4"
              class="mb-4"
            />

            <!-- Control buttons -->
            <v-row>
              <v-col>
                <v-btn
                  @click="initializeMpv"
                  color="primary"
                  :disabled="loading"
                  :loading="loading"
                  block
                >
                  Initialize MPV
                </v-btn>
              </v-col>

              <v-col>
                <v-btn
                  @click="loadVideo"
                  color="success"
                  :disabled="!isInitialized || !videoPath"
                  block
                >
                  Load Video
                </v-btn>
              </v-col>

              <v-col>
                <v-btn
                  @click="togglePlayPause"
                  color="warning"
                  :disabled="!isInitialized"
                  block
                >
                  Play/Pause
                </v-btn>
              </v-col>

              <v-col>
                <v-btn
                  @click="stopVideo"
                  color="error"
                  :disabled="!isInitialized"
                  block
                >
                  Stop
                </v-btn>
              </v-col>
            </v-row>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// Reactive state
const loading = ref(false);
const isInitialized = ref(false);
const videoPath = ref("");
const status = ref({
  type: "info",
  message: "Ready to initialize MPV player",
});

// Helper function to update status
const updateStatus = (type, message) => {
  status.value = { type, message };
  console.log(`[${type.toUpperCase()}]`, message);
};

// Initialize MPV player
const initializeMpv = async () => {
  loading.value = true;
  try {
    const result = await invoke("init_mpv_player");
    updateStatus("success", result);
    isInitialized.value = true;
  } catch (error) {
    updateStatus("error", `Failed to initialize: ${error}`);
    isInitialized.value = false;
  } finally {
    loading.value = false;
  }
};

// Load video file
const loadVideo = async () => {
  if (!videoPath.value) {
    updateStatus("warning", "Please enter a video file path");
    return;
  }

  try {
    const result = await invoke("load_video", { filePath: videoPath.value });
    updateStatus("success", result);
  } catch (error) {
    updateStatus("error", `Failed to load video: ${error}`);
  }
};

// Toggle play/pause
const togglePlayPause = async () => {
  try {
    const result = await invoke("play_pause");
    updateStatus("info", result);
  } catch (error) {
    updateStatus("error", `Failed to toggle playback: ${error}`);
  }
};

// Stop video
const stopVideo = async () => {
  try {
    const result = await invoke("stop_video");
    updateStatus("info", result);
  } catch (error) {
    updateStatus("error", `Failed to stop video: ${error}`);
  }
};
</script>
