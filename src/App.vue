<script setup lang="ts">
import { ref } from "vue";
import { exists, createDir, readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { appConfigDir} from "@tauri-apps/api/path";
import { Command, Child } from "@tauri-apps/api/shell";
import { listen } from "@tauri-apps/api/event";
import { process } from "@tauri-apps/api";

const states = {
  LOADING: "loading",
  SETTINGS: "settings",
  RUNNING: "running"
}

let isLoading = ref(true);
let isRunning = ref(false);
let isSettings = ref(false);
let config = ref({
  clientName: "Lyrion Minim",
  lyrionBaseUrl: "http://127.0.0.1:9000"
});
let iframeSrc = ref("about:blank");
let squeezeliteCommand: Command|null = null;
let squeezeliteProcess: Child|null = null;
let configFilePath = '';

// Settings menu item clicked
listen("settings", async () => {
  setState(states.SETTINGS);
});

// Quite menu item clicked
listen("quit", async () => {
  setState(states.LOADING);
  await stopSqueezelite();
  process.exit(0).then(console.log).catch(console.error).finally(console.log);
});

function startSqueezelite() {
  // Don't start it twice!
  if (squeezeliteCommand !== null) {
    return;
  }

  squeezeliteCommand = Command.sidecar("binaries/squeezelite", ["-n", config.value.clientName]);
  squeezeliteCommand.spawn()
    .then((process) => {
      squeezeliteProcess = process;
      console.log('Squeezelite process started ' + squeezeliteProcess.pid);
      
    })
    .catch(console.log)
    .finally(console.log);
}

async function stopSqueezelite() {
  if (squeezeliteProcess !== null) {
    await squeezeliteProcess.kill();
    console.log('Squeezelite process stopped ' + squeezeliteProcess.pid);
    squeezeliteProcess = null;
  }
}

function setState(value: string) {
  // If I knew what I was doing in Vue, this would suck less!
  isSettings.value = false;
  isLoading.value = false;
  isRunning.value = false;
  
  switch (value) {
    case states.SETTINGS:
      isSettings.value = true;
      break;
    case states.RUNNING:
      isRunning.value = true;
      break;
    case states.LOADING:
    default:
      isLoading.value = true;
  }
}

/**
 * Let's get started
 */

setState(states.LOADING);

( async () => {
  let configDir = await appConfigDir();
  configFilePath = configDir + "config.json";

  // Make configuration dir if it doesn't exist
  if (!await exists(configDir)) {
    await createDir(configDir, { recursive: true }).catch(console.error);
  }

  // Load configuration if it exists
  if (await exists(configFilePath)) {
    config.value = JSON.parse(await readTextFile(configFilePath));
    await showLyrion();
  }
})();



async function showLyrion() {

  startSqueezelite();
  iframeSrc.value = config.value.lyrionBaseUrl + "/Material/now-playing?single=1&player=" + config.value.clientName;
  setState(states.RUNNING);
}

async function saveConfig() {
  //await invoke("stop_squeezelite").catch(console.error);
  await writeTextFile(configFilePath, JSON.stringify(config.value, null, 2)).catch(console.error);
  await showLyrion();
}

</script>

<template>
  <div id="config" v-show="isSettings">
    <div class="row white-text">
      <h3>Setup Lyrion Minim</h3>
      <form class="col s12">
        <div class="row">
          <div class="input-field col s12">
            <input v-model="config.clientName" id="client-name" type="text" class="validate">
            <label for="client-name">Client Name</label>
            <span class="helper-text">The name of the client that will appear in Lyrion Music Server</span>
          </div>
        </div>
        <div class="row">
          <div class="input-field col s12">
            <input v-model="config.lyrionBaseUrl" id="lyrion-base-url" type="text" class="validate">
            <label for="lyrion-base-url">Lyrion Music Server Base URL</label>
            <span class="helper-text">Base URL for Lyrion Music Server e.g. http://127.0.0.1:9000</span>
          </div>
        </div>
        <div class="row">
          <div class="input-field col s12 center">
            <button class="btn waves-effect waves-light blue-grey" type="submit" name="action" @click="saveConfig">Save and Continue
              <i class="material-icons right">music_note</i>
            </button>
          </div>
        </div>
      </form>
    </div>
  </div>
  <iframe :src="iframeSrc" scrolling="no" v-show="isRunning"></iframe>
  <div id="splash" v-show="isLoading">
    <h1>test</h1>
    <div class="preloader-wrapper big active">
      <div class="spinner-layer spinner-blue-only">
        <div class="circle-clipper left">
          <div class="circle"></div>
        </div><div class="gap-patch">
          <div class="circle"></div>
        </div><div class="circle-clipper right">
          <div class="circle"></div>
        </div>
      </div>
    </div>
  </div>
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
</style>