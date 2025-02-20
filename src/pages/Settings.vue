<template>
  <div class="text-text flex w-full flex-col gap-8">
    <h5>Settings</h5>
    <div class="text-supporting flex flex-col gap-4">
      <p>Theme</p>
      <RadioButton
        @update:model-value="updateConfig(1)"
        v-model="theme"
        input-id="dark"
        input-value="Dark"
      />
      <RadioButton
        @update:model-value="updateConfig(1)"
        v-model="theme"
        input-id="light"
        input-value="Light"
      />
      <RadioButton
        @update:model-value="updateConfig(1)"
        v-model="theme"
        input-id="system"
        input-value="System"
      />
    </div>
    <div class="text-supporting">
      <p class="pb-4">Music Directory</p>
      <IconButton
        @click="openDialog"
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
    <div class="text-supporting">
      <p class="pb-4">Discord Rich Presence</p>
      <div class="flex gap-3">
        <input
          @change="updateConfig(4)"
          class="h-4 w-4"
          type="checkbox"
          v-model="discordRPC"
          id="discordRpc"
          name="discordRpc"
        />
        <label for="discordRpc">Enable Discord RPC</label>
      </div>
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
  usePlayerStore,
} from "@/composables/";
import { RadioButton, IconButton, Dialog } from "@/components/";
import { nextTick, onBeforeMount, ref } from "vue";

const configStore = useConfigStore();
const playerStore = usePlayerStore();

const theme = ref(configStore.config.theme);
const currentDirectory = ref(
  configStore.config?.music_dir || "No Folder Selected",
);
const lastFM = ref(configStore.config?.last_fm_key || "No Key Set");
const discordRPC = ref(configStore.config.discord_enabled);

function updateConfig(setting: number) {
  const updatedConfig: SodapopConfigEvent = {
    theme: null,
    music_dir: null,
    last_fm_key: null,
    discord_enabled: null,
  };

  switch (setting) {
    case 1:
      nextTick(() => {
        updatedConfig.theme = theme.value;
        configStore.config.theme = theme.value;
      });
      break;
    case 2:
      updatedConfig.music_dir = currentDirectory.value;
      configStore.config.music_dir = currentDirectory.value;
      break;
    case 3:
      updatedConfig.last_fm_key = lastFM.value;
      configStore.config.last_fm_key = lastFM.value;
      break;
    case 4:
      updatedConfig.discord_enabled = discordRPC.value;
      configStore.config.discord_enabled = discordRPC.value;
      break;
  }

  events.sodapopConfigEvent.emit(updatedConfig);
}

async function openDialog() {
  const result = await commands.selectMusicFolder();
  if (result.status === "error") return handleBackendError(result.error);
  else if (result.data !== "") {
    toastBus.addToast("success", "Music added successfully");
    currentDirectory.value = result.data;
    updateConfig(2);
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

onBeforeMount(async () => {
  playerStore.currentPage = "/settings";
  playerStore.pageName = "Settings";
});
</script>
