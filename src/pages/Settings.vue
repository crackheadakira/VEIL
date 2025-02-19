<template>
  <div class="text-text flex w-full flex-col gap-8">
    <h5>Settings</h5>
    <div class="text-supporting flex flex-col gap-4">
      <p>Theme</p>
      <RadioButton v-model="theme" input-id="dark" input-value="Dark" />
      <RadioButton v-model="theme" input-id="light" input-value="Light" />
      <RadioButton v-model="theme" input-id="system" input-value="System" />
    </div>
    <div class="text-supporting flex flex-col gap-4">
      <p>Music Directory</p>
      <IconButton
        @click="onClick"
        icon="i-fluent-folder-24-filled"
        :placeholder="currentDirectory"
      />
    </div>
    <div class="text-supporting flex flex-col gap-4">
      <p>Last.FM API Key</p>
      <Dialog
        title="Last.FM API Key"
        description="Enter your Last.FM API key to track your scrobbles"
      >
        <IconButton
          @click="onClick"
          icon="i-fluent-key-24-filled"
          :placeholder="lastFM"
        />
      </Dialog>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useConfigStore } from "@/composables/";
import { RadioButton, IconButton, Dialog } from "@/components/";
import { ref } from "vue";

const configStore = useConfigStore();

const theme = ref(configStore.config.theme);
const currentDirectory = ref(
  configStore.config?.music_dir || "No Folder Selected",
);
const lastFM = ref(configStore.config?.last_fm_key || "No Key Set");

function onClick() {
  console.log("Hello, World!");
}
</script>
