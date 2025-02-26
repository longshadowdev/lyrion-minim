<script setup lang="ts">
import { ref } from "vue";
import { Command, Child } from "@tauri-apps/api/shell";
import { listen } from "@tauri-apps/api/event";
import { process } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/tauri";

let iframeSrc = ref("/loading.html");
let debugMode = ref(false);
let configMode = ref(false);
let squeezeliteCommand: Command|null = null;
let squeezeliteProcess: Child|null = null;
let clientName = ref("Lyrion Minim");
let serverUrl = ref("");
let pid = ref(0);

// Quite menu item clicked
listen("quit", async () => {
  await stopSqueezelite();
  process.exit(0)
    .then(console.log)
    .catch(console.error)
    .finally(console.log);
});

// Debug menu item clicked
listen("debug", async () => {
  debugMode.value = !debugMode.value;
});

// Debug menu item clicked
listen("config", async () => {
  configMode.value = !configMode.value;
});

async function startSqueezelite() {
  console.log('Starting squeezelite');
  // Don't start it twice!
  if (squeezeliteProcess !== null) {
    return;
  }

  squeezeliteCommand = Command.sidecar("binaries/squeezelite", ["-n", clientName.value, "-M", "SqueezeLite" + getOS()]);
  squeezeliteProcess = await squeezeliteCommand.spawn();
  pid.value = squeezeliteProcess.pid;
  console.log('Squeezelite process started ' + squeezeliteProcess.pid);
}

async function stopSqueezelite() {
  if (squeezeliteProcess !== null) {
    console.log('Stopping squeezelite');
    await squeezeliteProcess.kill();
    console.log('Squeezelite process stopped ' + squeezeliteProcess.pid);
    squeezeliteProcess = null;
  }
}

function getOS() {
  const platform = window.navigator?.userAgentData?.platform || window.navigator.platform,
      macosPlatforms = ['macOS', 'Macintosh', 'MacIntel', 'MacPPC', 'Mac68K'],
      windowsPlatforms = ['Win32', 'Win64', 'Windows', 'WinCE'];
  let os = null;

  if (macosPlatforms.indexOf(platform) !== -1) {
    os = 'Mac';
  } else if (windowsPlatforms.indexOf(platform) !== -1) {
    os = 'Windows';
  } else if (/Linux/.test(platform)) {
    os = 'Linux';
  } else {
    os = '';
  }

  return os;
}

async function init() {
  console.log('Init');
  await startSqueezelite();
  serverUrl.value = await invoke("detect_lms_server");
  showPlayer();
}

function showPlayer() {
  iframeSrc.value = "http://" + serverUrl.value + "/Material/now-playing?player=" + clientName.value + "&layout=mobile";
}

async function saveConfig() {
  // await invoke("save_config", {
  //   clientName: clientName.value,
  //   serverUrl: serverUrl.value
  // });
  console.log(clientName.value);
  console.log(serverUrl.value);
  await stopSqueezelite();
  await startSqueezelite();
  showPlayer();
  configMode.value = false;
}

init();
/**
Config planning:

LMS Server:
- Auto detect
- Manually set

Squeezelite:
- Client Name (on change, restart squeezelite)
- Auto start / stop
- Stop button 
- Start button 
- Show current status (including PID)

*/
</script>

<template>
  <iframe :src="iframeSrc" scrolling="no" v-show="!configMode"></iframe>

  <div id="config" v-show="configMode" class="container" style="margin-top: 2em">
    
    <div class="row container" style="gap: 2em;">

      <div class="s12 m6 center">
        <img style="max-width: 20%; height:auto" src="/images/lyrion-logo.png">
        <h4>Lyrion Minim</h4>
        
        <h5>Configuration</h5>
      </div>

      <div class="s12 m6 input-field outlined">
        <input id="clientName" v-model="clientName" type="text" placeholder="Lyrion Minim" maxlength="20">
        <label for="clientName">Client Name</label>
        <span class="supporting-text">The name you want to give this client in Lyrion Music Server</span>
      </div>

      <div class="s12 m6 input-field outlined">
        <input id="serverUrl" type="text" v-model="serverUrl" placeholder="http://" maxlength="20">
        <label for="serverUrl">Server Base URL</label>
        <span class="supporting-text">Override the auto-detected servere URL. Format: host:port</span>
      </div>

      <div class="s12 m6 center">
        <button class="btn waves-effect waves-light blue-grey" name="action" @click="saveConfig">Save and Continue
          <i class="material-icons right">music_note</i>
        </button>
      </div>

    </div>
 
  </div>

  <div id="debug" v-show="debugMode">
    Pid: {{pid}}<br>
    URL: {{iframeSrc}}
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
iframe, #config {
  width: 100%;
  height: 100%;
  border: none;
  padding: 0;
  margin: 0;
  overflow: hidden;
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
  overflow: hidden;
}
</style>