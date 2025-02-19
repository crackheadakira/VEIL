<template>
  <div class="text-text flex w-full flex-col gap-8">
    <h5>Settings</h5>
    <div class="text-supporting flex flex-col gap-4">
      <p>Theme</p>
      <RadioButton
        @vue:updated="updateConfig(1, theme)"
        v-model="theme"
        input-id="dark"
        input-value="Dark"
      />
      <RadioButton v-model="theme" input-id="light" input-value="Light" />
      <RadioButton v-model="theme" input-id="system" input-value="System" />
    </div>
    <div @click="openDialog" class="text-supporting">
      <p class="pb-4">Music Directory</p>
      <IconButton
        icon="i-fluent-folder-24-filled"
        :placeholder="currentDirectory"
      />
    </div>
    <div class="text-supporting">
      <p class="pb-4">Last.FM API Key</p>
      <Dialog
        title="Last.FM API Key"
        description="Enter your Last.FM API key to track your scrobbles"
      >
        <IconButton icon="i-fluent-key-24-filled" :placeholder="lastFM" />
      </Dialog>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  useConfigStore,
  handleBackendError,
  toastBus,
  commands,
  events,
  SodapopConfigEvent,
} from "@/composables/";
import { RadioButton, IconButton, Dialog } from "@/components/";
import { ref } from "vue";

const configStore = useConfigStore();

const theme = ref(configStore.config.theme);
const currentDirectory = ref(
  configStore.config?.music_dir || "No Folder Selected",
);
const lastFM = ref(configStore.config?.last_fm_key || "No Key Set");

function updateConfig(setting: number, value: any) {
  const updatedConfig: SodapopConfigEvent = {
    theme: null,
    music_dir: null,
    last_fm_key: null,
    discord_enabled: null,
  };

  switch (setting) {
    case 1:
      updatedConfig.theme = value;
      configStore.config.theme = value;
      console.log(configStore.config.theme);
      break;
    case 2:
      updatedConfig.music_dir = value;
      configStore.config.music_dir = value;
      break;
    case 3:
      updatedConfig.last_fm_key = value;
      configStore.config.last_fm_key = value;
      break;
    case 4:
      updatedConfig.discord_enabled = value;
      configStore.config.discord_enabled = value;
      break;
  }

  events.sodapopConfigEvent.emit(updatedConfig);
}

async function openDialog() {
  const result = await commands.selectMusicFolder();
  if (result.status === "error") return handleBackendError(result.error);
  else if (result.data !== "") {
    toastBus.addToast("success", "Music added successfully");
    updateConfig(2, result.data);
    currentDirectory.value = result.data;
  }
}

const persistentToastId = ref<number | null>(null);

events.musicDataEvent.once((data) => {
  const id = Date.now();
  persistentToastId.value = id;
  const payload = data.payload;
  // keep modfying the toast description until the data.finished is true
  toastBus.persistentToast(
    id,
    "info",
    `Importing songs (${payload.current} / ${payload.total})`,
  );

  if (payload.finished) {
    setTimeout(() => toastBus.removeToast(id), 2100);
  }
});

events.musicDataEvent.listen((data) => {
  const payload = data.payload;

  if (persistentToastId.value) {
    toastBus.persistentToast(
      persistentToastId.value,
      "info",
      `Importing songs (${payload.current} / ${payload.total})`,
    );
  } else {
    const id = Date.now();
    persistentToastId.value = id;
    toastBus.persistentToast(
      id,
      "info",
      `Importing songs (${payload.current} / ${payload.total})`,
    );
  }

  if (payload.finished && persistentToastId.value) {
    setTimeout(() => {
      toastBus.removeToast(persistentToastId.value!);
      persistentToastId.value = null;
    }, 2100);
  }
});
</script>
