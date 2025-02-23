<template>
  <div>
    <Transition
      enter-from-class="opacity-0 scale-95"
      leave-to-class="opacity-0 scale-95"
      enter-active-class="transition-all duration-150"
      leave-active-class="transition-all duration-150"
    >
      <div
        v-if="showDialog"
        class="bg-background/50 absolute inset-0 z-50 flex items-center justify-center"
      >
        <div class="text-text flex h-72 w-96 flex-col">
          <div
            class="bg-background border-stroke-100 flex w-full items-center gap-2 rounded-md border p-2 font-medium"
          >
            <span
              class="i-fluent-search-12-filled text-supporting aspect-square w-5"
            ></span>
            <input
              @update:model-value="getResults"
              v-model="inputValue"
              ref="input"
              type="text"
              class="placeholder-supporting w-full focus:outline-hidden"
              placeholder="Search..."
            />
          </div>

          <Transition
            enter-from-class="opacity-0 -translate-y-2"
            leave-to-class="opacity-0 -translate-y-2"
            enter-active-class="transition-all duration-150"
            leave-active-class="transition-all duration-150"
          >
            <div
              v-if="searchResults && searchResults.length"
              class="border-stroke-100 bg-card flex max-h-64 flex-col gap-2 overflow-scroll border border-t-0 p-2"
            >
              <RouterLink
                :key="result.title + result.search_id"
                :to="{
                  name: result.search_type,
                  params: { id: result.search_id },
                }"
                @click="showDialog = false"
                v-for="result of searchResults"
                class="hover:bg-background transition-color flex w-full cursor-pointer items-center justify-between gap-2 rounded-md p-3 duration-75"
              >
                <small class="truncate">{{ result.title }}</small>
                <small class="text-supporting shrink-0">{{
                  readableCapitalization(result.search_type)
                }}</small>
              </RouterLink>
            </div>
          </Transition>
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

  await new Promise((r) => setTimeout(r, 250));

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
