<template>
  <v-app>
    <v-layout>
      <v-app-bar color="primary" prominent>
        <v-app-bar-title>Tauri MPV Media Player</v-app-bar-title>
      </v-app-bar>

      <v-main>
        <MpvPlayer />
      </v-main>
    </v-layout>
  </v-app>
</template>

<script setup>
import MpvPlayer from "./components/MpvPlayer.vue";
</script>

<style scoped>
/* Add any global styles here */
</style>
