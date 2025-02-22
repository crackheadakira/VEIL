<template>
  <div>
    <div
      v-if="showDialog"
      class="bg-background/50 absolute top-0 left-0 z-40 h-screen w-screen"
    ></div>
    <Transition
      enter-from-class="opacity-0 scale-95"
      leave-to-class="opacity-0 scale-95"
      enter-active-class="transition-all duration-150"
      leave-active-class="transition-all duration-150"
    >
      <div
        v-if="showDialog"
        class="absolute inset-0 z-50 flex items-center justify-center"
      >
        <div
          class="cardStyle text-text relative flex h-fit w-96 flex-col overflow-hidden p-0"
        >
          <div
            class="cardStyle bg-background flex w-full items-center gap-2 rounded-none border-0 font-medium"
          >
            <span
              class="i-fluent-search-12-filled text-supporting aspect-square w-5"
            ></span>
            <input
              @update:model-value="getResults"
              v-model="inputValue"
              ref="input"
              type="text"
              class="placeholder-supporting focus:outline-hidden"
              placeholder="Search..."
            />
          </div>
          <div
            v-if="searchResults && searchResults.length"
            class="border-stroke-100 flex flex-col gap-2 border-t p-2"
          >
            <RouterLink
              :to="{
                name: result.search_type,
                params: { id: result.search_id },
              }"
              @click="showDialog = false"
              v-for="result of searchResults"
              class="hover:bg-background flex w-full cursor-pointer items-center justify-between rounded-md p-3 transition duration-75"
            >
              <small>{{ result.title }}</small>
              <small class="text-supporting">{{
                readableCapitalization(result.search_type)
              }}</small>
            </RouterLink>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { Search, commands, readableCapitalization } from "@/composables/";
import { templateRef, useEventListener } from "@vueuse/core";

const input = templateRef("input");
const showDialog = ref(false);
const inputValue = ref("");
const lastInput = ref(Date.now());

const searchResults = ref<Search[] | null>(null);

async function getResults() {
  const calledAt = Date.now();
  lastInput.value = Date.now();

  await new Promise((r) => setTimeout(r, 500));

  if (calledAt != lastInput.value) return;

  if (!inputValue.value.length) {
    searchResults.value = null;
    return;
  }

  const result = await commands.searchDb(inputValue.value);
  if (result.status === "error") return console.log(result.error);

  searchResults.value = result.data;
}

/**
 * Handle the keydown event.
 *
 * If the `Escape` key is pressed, close the dialog.
 */
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    showDialog.value = false;
  }
}

useEventListener("keydown", (e) => {
  if (e.ctrlKey && e.key === "f") {
    showDialog.value = !showDialog.value;
    nextTick(() => {
      input.value.focus();
    });
  }
});

watch(showDialog, (value) => {
  if (value) {
    window.addEventListener("keydown", handleKeyDown);
  } else {
    window.removeEventListener("keydown", handleKeyDown);
  }
});
</script>
