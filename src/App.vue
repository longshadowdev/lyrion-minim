<script setup lang="ts">
import { ref } from "vue";
import { Command, Child } from "@tauri-apps/api/shell";
import { listen } from "@tauri-apps/api/event";
import { process } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/tauri";

let iframeSrc = ref("loading.html");
let debug = ref(true);
let squeezeliteCommand: Command|null = null;
let squeezeliteProcess: Child|null = null;
let clientName = "Lyrion Minim";
let pid = ref(0);

// Quite menu item clicked
listen("quit", async () => {
  await stopSqueezelite();
  process.exit(0).then(console.log).catch(console.error).finally(console.log);
});

// Debug menu item clicked
listen("debug", async () => {
  debug.value = !debug.value;
});

function sleep(seconds: number) {
  return new Promise((resolve) => setTimeout(resolve, seconds * 1000));
}

async function startSqueezelite() {
  // Don't start it twice!
  if (squeezeliteCommand !== null) {
    return;
  }

  squeezeliteCommand = Command.sidecar("binaries/squeezelite", ["-n", clientName]);
  squeezeliteProcess = await squeezeliteCommand.spawn();
  pid.value = squeezeliteProcess.pid;
  console.log('Squeezelite process started ' + squeezeliteProcess.pid);
}

async function stopSqueezelite() {
  if (squeezeliteProcess !== null) {
    await squeezeliteProcess.kill();
    console.log('Squeezelite process stopped ' + squeezeliteProcess.pid);
    squeezeliteProcess = null;
  }
}

async function init() {
  await startSqueezelite();
  let lmsServer = await invoke("detect_lms_server");
  iframeSrc.value = "http://" + lmsServer + "/Material/now-playing?player=" + clientName;
}


init();

</script>

<template>
  <iframe :src="iframeSrc" scrolling="no"></iframe>
  <div id="debug" v-show="debug">Pid: {{pid}}<br> URL: {{iframeSrc}}</div>
</template>

<style>
body {
  opacity: 0.95;
}
html, body, #app {
  margin: 0; 
  height: 100%; 
  overflow: hidden;
  border-radius: 0.8em;
}
iframe, #config, #splash {
  width: 100%;
  height: 100%;
  border: none;
  padding: 0;
  margin: 0;
  overflow: hidden;
}
#config, #splash {
  padding: 2em;
  background-color: black;
  opacity: 0.85;
  color: white;
}
input {
  color:white;
}
#debug {
  font-family: Arial, Helvetica, sans-serif;
  position: fixed;
  margin-top: -2.5em;
  background-color: black;
  color: white;
  font-size: 0.7em;
  z-index: 9;
}
</style>