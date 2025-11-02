<template>
  <div class="text-text-primary flex w-full flex-col gap-8">
    <h5>Settings</h5>
    <div class="text-text-secondary flex flex-col gap-4">
      <p>Theme</p>
      <RadioGroup
        @update:model-value="(e: string) => updateConfig(1, e)"
        v-model="theme as ThemeMode"
        :items="['Dark', 'Light', 'System']"
      />
    </div>
    <div class="text-text-secondary">
      <p class="pb-4">Music Directory</p>
      <IconButton
        @click="openDialog"
        icon="i-fluent-folder-24-filled"
        :placeholder="currentDirectory"
      />
    </div>
    <div class="text-text-secondary flex flex-col gap-4">
      <p>Online Features</p>
      <div class="flex gap-3">
        <Switch
          @update:model-value="(e: boolean) => updateConfig(4, e)"
          v-model="discordRPC"
          id="discordRPC"
        />
        <label for="discordRPC">Discord RPC</label>
      </div>
      <div class="flex gap-3">
        <Switch
          @update:model-value="(e: boolean) => updateConfig(5, e)"
          v-model="lastFM"
          id="lastFM"
        />
        <label for="lastFM">Last.FM</label>
      </div>
      <div v-if="lastFM">
        <p class="pb-2">Last.FM Session Key</p>
        <DialogGuide @close="pageIdx = 0" :current-page="pages[pageIdx]">
          <IconButton icon="i-fluent-key-24-filled" :placeholder="lastFMKey" />
        </DialogGuide>
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
  DialogPage,
  MetadataEvent,
  ThemeMode,
} from "@/composables/";
import { IconButton, DialogGuide, Switch, RadioGroup } from "@/components/";
import { computed, ComputedRef, nextTick, onBeforeMount, ref } from "vue";
import { Channel } from "@tauri-apps/api/core";

const onEvent = new Channel<MetadataEvent>();
const configStore = useConfigStore();

const theme = ref(configStore.config.ui.theme);
const currentDirectory = ref(
  configStore.config.library.music_dir || "No Folder Selected",
);
const lastFMKey = ref(
  configStore.config.integrations.last_fm_session_key || "No Key Set",
);
const discordRPC = ref(
  configStore.config.integrations.discord_enabled ?? false,
);
const lastFM = ref(configStore.config.integrations.last_fm_enabled ?? false);

const lastFMURL = ref<[string, string] | null>(null);

const pageIdx = ref(0);
const pages: ComputedRef<DialogPage[]> = computed(() => [
  {
    title: "Last.FM",
    description: "Manage your Last.FM session",
    buttons: [
      {
        name: "revoke",
        condition:
          configStore.config.integrations.last_fm_session_key?.length !== 0,
        close: true,
        click: async () => {
          updateConfig(3, "");
          updateConfig(5, false);

          lastFM.value = false;
          lastFMKey.value = "No Key Set";
        },
      },
      {
        name: "start",
        condition:
          configStore.config.integrations.last_fm_session_key?.length === 0,
        click: getToken,
      },
    ],
  },
  {
    title: "Authorization",
    description: "Once you've authorized on Last.FM, press continue.",
    buttons: [
      {
        name: "continue",
        condition: true,
        close: true,
        click: registerSession,
      },
    ],
  },
]);

function updateConfig(setting: number, value: any) {
  const updatedConfig = {
    theme: null,
    music_dir: null,
    last_fm_session_key: null,
    discord_enabled: null,
    last_fm_enabled: null,
    queue_origin: null,
    queue_idx: null,
    repeat_mode: null,
  } satisfies SodapopConfigEvent;

  switch (setting) {
    case 1:
      updatedConfig.theme = value;
      configStore.config.ui.theme = value;
      break;
    case 2:
      updatedConfig.music_dir = value;
      configStore.config.library.music_dir = value;
      break;
    case 3:
      updatedConfig.last_fm_session_key = value;
      configStore.config.integrations.last_fm_session_key = value;
      break;
    case 4:
      updatedConfig.discord_enabled = value;
      configStore.config.integrations.discord_enabled = value;
      break;
    case 5:
      updatedConfig.last_fm_enabled = value;
      configStore.config.integrations.last_fm_enabled = value;
      break;
  }

  events.sodapopConfigEvent.emit(updatedConfig);
}

async function getToken() {
  const result = await commands.getToken();
  if (result.status === "error") return handleBackendError(result.error);

  lastFMURL.value = result.data;
  // await openUrl(result.data[0]);
  await commands.openUrl(result.data[0]);
  pageIdx.value = 1;
}

async function registerSession() {
  if (!lastFMURL.value) return;
  const result = await commands.getSession(lastFMURL.value[1]);
  if (result.status === "error") return handleBackendError(result.error);

  toastBus.addToast("success", "Successfully registered Last.FM session!");
  await configStore.initialize();

  nextTick(() => {
    lastFMKey.value = configStore.config.integrations.last_fm_session_key!;
  });
}

async function openDialog() {
  const result = await commands.selectMusicFolder(onEvent);
  if (result.status === "error") return handleBackendError(result.error);
  else if (result.data !== "") {
    toastBus.addToast("success", "Music added successfully");
    currentDirectory.value = result.data;
    updateConfig(2, result.data);
  }
}

const persistentToastId = ref<number | null>(null);
const totalSongs = ref<number | null>(null);

onEvent.onmessage = (res) => {
  if (res.event === "Started") {
    persistentToastId.value = res.data.id;
    totalSongs.value = res.data.total;

    toastBus.persistentToast(
      res.data.id,
      "info",
      `Going to import ${res.data.total} songs!`,
    );
  } else if (res.event === "Progress") {
    toastBus.persistentToast(
      res.data.id,
      "info",
      `Importing songs (${res.data.current} / ${totalSongs.value})`,
    );
  } else {
    setTimeout(() => {
      toastBus.removeToast(res.data.id);
      persistentToastId.value = null;
      totalSongs.value = null;
    });
  }
};

onBeforeMount(async () => {
  configStore.currentPage = "/settings";
  configStore.pageName = "Settings";
});
</script>
