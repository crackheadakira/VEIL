<template>
  <div class="text-text flex w-full flex-col gap-8">
    <h5>Settings</h5>
    <div>
      <p>Theme</p>
      <input type="radio" id="dark" value="Dark" v-model="picked" />
      <label for="dark">Dark</label>

      <input type="radio" id="light" value="Light" v-model="picked" />
      <label for="light">Light</label>

      <input type="radio" id="system" value="System" v-model="picked" />
      <label for="system">System</label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useConfigStore } from "@/composables/";

const configStore = useConfigStore();

const picked = ref(configStore.config.theme);
</script>
