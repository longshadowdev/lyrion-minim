<script setup lang="ts">
import { ref } from "vue";
import { Command, Child } from "@tauri-apps/api/shell";
import { listen } from "@tauri-apps/api/event";
import { process } from "@tauri-apps/api";
import { invoke } from "@tauri-apps/api/tauri";
import { getVersion } from "@tauri-apps/api/app";
import { exists, BaseDirectory, createDir, readTextFile, writeTextFile } from "@tauri-apps/api/fs";
import { appConfigDir } from "@tauri-apps/api/path";
import { platform, Platform } from "@tauri-apps/api/os";
import { http } from "@tauri-apps/api";

const fileSpecVersion = 1;

// State including config
const state = ref({
  version: '',
  iframeSrc: 'about:blank',
  configMode: true,
  configLoaded: false,
  config: {
    fileSpecVersion: fileSpecVersion,
    client: {
      autostart: true,
      name: 'Lyrion Minim',
    },
    server: {
      autodetect: true,
      host: '',
      port: 9000,
    }
  },
  client: {
    running: false,
    needsRestart: false,
    squeezeliteProcess: <Child|null>(null)
  },
  server: {
    loading: false
  }
});

// Quite menu item clicked
listen('quit', async () => {
  await client.stop();
  await process.exit(0);
});

// Debug menu item clicked
listen('config', async () => {
  state.value.configMode = !state.value.configMode;
});

// Config Manager
let configManager = {
  configFileName: 'config.json',
  load: async () => {
    const loadedConfig = await readTextFile(configManager.configFileName, { dir: BaseDirectory.AppConfig })
      .then((data) => JSON.parse(data))
      .catch(() => false);
    
    if (loadedConfig && loadedConfig.fileSpecVersion === fileSpecVersion) {
      state.value.configMode = false;
      state.value.configLoaded = true;
      state.value.config = loadedConfig;
    }
  },
  save: async () => {
    await configManager.createConfigDir();
    await writeTextFile(
      configManager.configFileName, 
      JSON.stringify(state.value.config), 
      { dir: BaseDirectory.AppConfig }
    ).catch((err) => messager.show('Unable to save configuration: ' + err));
  },
  createConfigDir: async () => {
    const configDir = await appConfigDir();
    if (!await exists(configDir)) {
      await createDir(await appConfigDir());
    }
  },
  handleSave: async () => {
    if (!await server.validate()) {
      messager.show('Server is not reachable. Please ensure it is running and has the Material skin installed', 2000);
      return;
    }

    await configManager.save();

    if (state.value.client.needsRestart && state.value.client.running) {
      await client.restart();
      state.value.client.needsRestart = false;
    }
    
    server.showPlayer();

    state.value.configMode = false;
  }
}

// Squeezelite Client Management
let client = {
  start: async (silent: boolean = false) => {
    // Don't start it twice!
    if (state.value.client.squeezeliteProcess !== null) {
      return;
    }
    !silent ? messager.show('Starting Squeezelite') : null;
    let squeezeliteCommand = Command.sidecar('binaries/squeezelite', [
      '-n', state.value.config.client.name, 
      '-M', await client.getClientType()
    ]);
    state.value.client.squeezeliteProcess = await squeezeliteCommand.spawn()
      .then((process) => {
        state.value.client.running = true;
        !silent ? messager.show('Squeezelite process started ' + process.pid) : null;
        return process;
      })
      .catch((err) => {
        messager.show('Unable to start Squeezelite ' + err);
        console.log(err);
        return null;
      });
    
  },
  stop: async (silent: boolean = false) => {
    if (state.value.client.squeezeliteProcess !== null) {
    !silent ? messager.show('Stopping Squeezelite') : null;
    await state.value.client.squeezeliteProcess.kill();
    !silent ? messager.show('Squeezelite process stopped') : null;
    state.value.client.squeezeliteProcess = null;
  }
  },
  toggle: async (enabled: boolean) => {
    enabled ? await client.start() : await client.stop();
  },
  restart: async () => {
    messager.show('Restarting Squeezelite');
    await client.stop(true);
    await client.start(true);
  },
  getClientType: async () => {
    const platformName: Platform = await platform();
    const map = {
      'linux': 'Linux',
      'darwin': 'Mac',
      'ios': 'Mac',
      'freebsd': '',
      'dragonfly': '',
      'netbsd': '',
      'openbsd': '',
      'solaris': '',
      'android': '',
      'win32': 'Windows',
    };
    return 'SqueezeLite' + (map[platformName] ?? '');
  },
  handleToggle: async (item: Event) => {
    ((<HTMLInputElement>item.target).checked)
      ? await client.start() 
      : await client.stop();

    return true;
  }
}

// Snackbar message state
const messageState = ref({
  snackbar: false,
  snackbarText: '',
  timeout: 1000
});
let messager = {
  show: (text: string, timeout: number = 1000) => {
    messager.hide();
    messageState.value.timeout = timeout;
    messageState.value.snackbarText = text;
    messageState.value.snackbar = true;
  },
  hide: () => {
    messageState.value.snackbarText = '';
    messageState.value.snackbar = false;
  }
}

// Manage everything to do with the LMS server
let server = {
  detect: async () => {
    if (!state.value.config.server.autodetect) return;
    state.value.server.loading = true;
    messager.show('Autodetecting server');
    let server: { host: string, port: number } = await invoke('detect_lms_server');
    if (server?.host === undefined) {
      messager.show('Unable to autodetect server. Please enter server manually.', 2000);
      state.value.config.server.autodetect = false;
    }
    state.value.config.server.host = server.host ?? state.value.config.server.host;
    state.value.config.server.port = server.port ?? state.value.config.server.port;
    state.value.server.loading = false;
  },
  validate: async () => {
    // Make sure the server is reachable and has Material installed
    return await http.fetch(server.getBaseUri(), { method: 'HEAD'})
      .then(response => response.status === 200)
      .catch(() => false);
  },
  showPlayer: () => {
    state.value.iframeSrc = server.getUri();
  },
  getBaseUri: () => {
    return "http://" 
        + state.value.config.server.host 
        + ":" 
        + state.value.config.server.port 
        + "/Material";
  },
  getUri: () => {
    return server.getBaseUri() 
        + "/now-playing?"
        + "player=" + state.value.config.client.name 
        + "&layout=mobile";
  }
}

async function init() {

  console.log('Init');
  state.value.version = await getVersion();
  await configManager.load();

  if (state.value.config.client.autostart) {
    await client.start();
  }
  
  if (state.value.config.server.autodetect) {
    await server.detect();
  }

  if (state.value.configLoaded) {
    // Only show player if config has loaded from file. Otherwise the config
    // screen would be shown
    server.showPlayer();
  }
}

init();
</script>

<template>
  <iframe :src="state.iframeSrc" scrolling="no" v-show="!state.configMode"></iframe>

  <v-container id="config" class="bg-surface-variant" v-show="state.configMode">
    <v-sheet class="pa-6 text-white mx-auto fill-height" >

      <v-card
        class="mb-5"
        border
        append-avatar="/images/lyrion-logo.png"
        :subtitle="'Version ' + state.version "
        variant="tonal"
      >
        <template v-slot:title>
          <span class="font-weight-black text-h5">Lyrion Minim</span>
        </template>
      </v-card>

      <v-card
        class="mb-5"
        border
        subtitle="Configure the built-in Squeezelite client"
        title="Client Settings"
      >
        <v-row class="mt-1">
          <v-col class="mx-4 pa-0">
            <v-switch
              class="mx-4"
              v-model="state.client.running"
              @change="client.handleToggle"
              :label="state.client.squeezeliteProcess === null ? 'Stopped' : 'Started'"
              hide-details
            ></v-switch>
          </v-col>
          <v-col class="mx-4 pa-0">
          <v-switch
            class="mx-4"
            v-model="state.config.client.autostart"
            label="Auto start"
            hide-details
          ></v-switch>
          </v-col>
        </v-row>
        <v-row class="pb-1">
          <v-col>
            <v-text-field 
              class="mx-4" 
              label="Client Name" 
              placeholder="Lyrion Minim" 
              v-model="state.config.client.name"
              v-on:update:model-value="state.client.needsRestart = true"
              hint="How this client will appear in Lyrion Music Server"
            ></v-text-field>
          </v-col>
        </v-row>
      </v-card>

      <v-card
        class="mb-5"
        border
        subtitle="Configure the connection to Lyrion Music Server"
        title="Server Settings"
        :loading="state.server.loading"
      >
        <v-row class="mt-1">
          <v-col class="mx-4 pa-0">
            <v-switch
              class="mx-4"
              v-model="state.config.server.autodetect"
              @change="server.detect"
              label="Auto detect server"
              hide-details
              :disabled="state.server.loading"
            ></v-switch>
          </v-col>
        </v-row>
        <v-row class="pb-1">
          <v-col>
            <v-text-field 
              class="ml-4"
              :disabled="state.config.server.autodetect"
              label="Host" 
              placeholder="127.0.0.1" 
              v-model="state.config.server.host" 
              hint="Hostname or IP"
            ></v-text-field>
          </v-col>
          <v-col cols="5">
            <v-text-field 
              class="mr-4"
              :disabled="state.config.server.autodetect"
              label="Port" 
              placeholder="9000" 
              v-model="state.config.server.port" 
              hint="Network port"
            ></v-text-field>
          </v-col>
        </v-row>
      </v-card>
      <v-btn
        size="large"
        type="submit"
        variant="tonal"
        block
        @click="configManager.handleSave"
      >
        Save and continue
      </v-btn>

    </v-sheet>
  </v-container>

  <v-snackbar v-model="messageState.snackbar" :timeout="messageState.timeout">{{messageState.snackbarText}}</v-snackbar>

</template>

<style>
body {
  opacity: 0.97;
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
</style>