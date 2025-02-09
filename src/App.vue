<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const config = ref();
(async () => {
  config.value = await invoke("get_config");
  config.value.lyrionUrl = config.value.lyrionBaseUrl + "/Material/now-playing?single=1&player=" + config.value.clientName;
})();

</script>

<template>
  <iframe :src="config.lyrionUrl" scrolling="no"></iframe>
</template>

<style>
body {
  opacity: 0.95;
}
html, body, #app {
  margin: 0; 
  height: 100%; 
  overflow: hidden;
  border-radius: 0.5em;
}
iframe {
  width: 100%;
  height: 100%;
  border: none;
  padding: 0;
  margin: 0;
  overflow: hidden;
}
</style>