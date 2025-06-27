<template>
  <v-container>
    <v-row>
      <!-- Left Panel: Controls -->
      <v-col cols="12" md="4">
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
              placeholder="/Users/yourname/Movies/video.mp4"
              hint="Enter full path to a video file on your system"
              class="mb-4"
            />

            <!-- Initialize button -->
            <v-btn
              @click="initializeMpv"
              color="primary"
              :disabled="loading"
              :loading="loading"
              block
              class="mb-4"
            >
              Initialize MPV
            </v-btn>

            <!-- Load video button -->
            <v-btn
              @click="loadVideo"
              color="success"
              :disabled="!isInitialized || !videoPath"
              block
              class="mb-4"
            >
              Load Video
            </v-btn>

            <!-- Media Controls Component -->
            <MediaControls
              :is-initialized="isInitialized"
              @status-update="updateStatus"
            />

            <!-- Instructions -->
            <v-card class="mt-4" variant="outlined">
              <v-card-text>
                <h4>Instructions:</h4>
                <ol>
                  <li>Click "Initialize MPV" first</li>
                  <li>Enter the full path to a video file</li>
                  <li>Click "Load Video"</li>
                  <li>Use the media controls</li>
                  <li>Video should appear on the right →</li>
                </ol>
              </v-card-text>
            </v-card>
          </v-card-text>
        </v-card>
      </v-col>

      <!-- Right Panel: Video Area -->
      <v-col cols="12" md="8">
        <v-card height="600">
          <v-card-title>Video Output</v-card-title>
          <v-card-text class="pa-0">
            <!-- Video rendering area -->
            <div
              ref="videoContainer"
              id="mpv-video-container"
              class="video-container"
              :style="{
                width: '100%',
                height: '550px',
                backgroundColor: '#000',
                position: 'relative',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
              }"
            >
              <!-- Placeholder content -->
              <div v-if="!isInitialized" class="text-center text-grey">
                <v-icon size="64" color="grey">mdi-video-off</v-icon>
                <p class="mt-2">Initialize MPV to enable video</p>
              </div>

              <div v-else-if="!videoLoaded" class="text-center text-grey">
                <v-icon size="64" color="grey">mdi-video-plus</v-icon>
                <p class="mt-2">Load a video file to start playback</p>
              </div>

              <div v-else class="text-center text-white">
                <v-icon size="64" color="white">mdi-video</v-icon>
                <p class="mt-2">Video should render here</p>
                <p class="text-caption">
                  If you only hear audio, check the console for errors
                </p>
              </div>

              <!-- Future: HTML overlay for custom subtitles will go here -->
              <div
                v-if="videoLoaded"
                class="subtitle-overlay"
                :style="{
                  position: 'absolute',
                  bottom: '20px',
                  left: '50%',
                  transform: 'translateX(-50%)',
                  backgroundColor: 'rgba(0,0,0,0.7)',
                  color: 'white',
                  padding: '8px 16px',
                  borderRadius: '4px',
                  zIndex: 1000,
                }"
              >
                Future: Custom subtitle overlay will appear here
              </div>
            </div>
          </v-card-text>
        </v-card>
      </v-col>
    </v-row>
  </v-container>
</template>

<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import MediaControls from "./MediaControls.vue";

// Reactive state
const loading = ref(false);
const isInitialized = ref(false);
const videoLoaded = ref(false);
const videoPath = ref("");
const videoContainer = ref(null);
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
    // First setup video rendering area
    await setupVideoRendering();

    // Then load the video
    const result = await invoke("load_video", { filePath: videoPath.value });
    updateStatus("success", result);
    videoLoaded.value = true;
  } catch (error) {
    updateStatus("error", `Failed to load video: ${error}`);
    videoLoaded.value = false;
  }
};

// Setup video rendering in the designated area
const setupVideoRendering = async () => {
  if (!videoContainer.value) {
    throw new Error("Video container not found");
  }

  // Get the video container's position and size
  const rect = videoContainer.value.getBoundingClientRect();
  const videoArea = {
    x: Math.round(rect.left),
    y: Math.round(rect.top),
    width: Math.round(rect.width),
    height: Math.round(rect.height),
  };

  console.log("Setting up video rendering at:", videoArea);

  try {
    const result = await invoke("setup_video_rendering", { videoArea });
    updateStatus("info", result);
  } catch (error) {
    throw new Error(`Failed to setup video rendering: ${error}`);
  }
};

onMounted(() => {
  // When we implement native rendering, we'll get the container element here
  console.log("Video container mounted:", videoContainer.value);
});
</script>

<style scoped>
.video-container {
  border: 2px dashed #666;
}

.subtitle-overlay {
  font-family: "Arial", sans-serif;
  font-size: 16px;
  text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.8);
}
</style>
