<template>
  <div
    @click="updateDialog"
    class="text-supporting sodapop-card bg-background hover:border-stroke-100 flex w-full cursor-pointer items-center gap-2 p-3 duration-150"
  >
    <span class="i-fluent-search-20-filled"></span>
    <small>Search...</small>
    <small class="text-text bg-stroke-100 ml-auto rounded-sm p-1 px-2"
      >Ctrl F</small
    >
  </div>
  <Teleport to="body">
    <div>
      <Transition
        enter-active-class="animate-zoomIn"
        leave-active-class="animate-zoomOut"
      >
        <div
          v-if="showDialog"
          class="bg-background/50 absolute inset-0 z-50 flex items-center justify-center"
        >
          <div class="text-text flex h-72 w-96 flex-col">
            <div
              class="bg-background border-stroke-200 flex w-full items-center gap-2 rounded-md border p-2 font-medium"
            >
              <span
                class="i-fluent-search-12-filled text-supporting aspect-square w-5"
              ></span>
              <input
                v-model="input"
                ref="inputElement"
                type="text"
                @focusin="focused = true"
                @focusout="focused = false"
                class="placeholder-supporting w-full focus:outline-hidden"
                placeholder="Search..."
              />
            </div>

            <Transition
              enter-active-class="animate-slideDownAndFade"
              leave-active-class="animate-slideDownAndFade animation-reverse"
            >
              <div
                v-if="searchResults && searchResults.length"
                class="border-stroke-200 bg-card flex max-h-64 flex-col gap-2 overflow-scroll border border-t-0 p-2"
              >
                <div
                  :key="result.title + result.search_id"
                  @click="
                    ((showDialog = false),
                    router.push(`/${result.search_type}/${result.search_id}`))
                  "
                  v-for="(result, idx) of searchResults"
                  :class="idx === selected ? 'bg-background' : ''"
                  ref="resultElements"
                  class="hover:bg-background transition-color flex w-full cursor-pointer items-center justify-between gap-2 rounded-md p-3 duration-75"
                >
                  <small class="truncate">{{ result.title }}</small>
                  <small class="text-supporting shrink-0">{{
                    readableCapitalization(result.search_type)
                  }}</small>
                </div>
              </div>
            </Transition>
          </div>
        </div>
      </Transition>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { Search, commands, readableCapitalization } from "@/composables/";
import { templateRef, useEventListener } from "@vueuse/core";
import { useRouter } from "vue-router";

const router = useRouter();

const clamp = (v: number) =>
  Math.max(
    0,
    Math.min(searchResults.value ? searchResults.value.length - 1 : 1, v),
  );
const showDialog = ref(false);
const lastInput = ref(Date.now());

const inputElement = templateRef("inputElement");
const input = ref("");
const selected = ref(0);
const focused = ref(false);

const searchResults = ref<Search[] | null>(null);
const resultElements = templateRef("resultElements");

watch(input, async () => {
  const calledAt = Date.now();
  lastInput.value = Date.now();

  await new Promise((r) => setTimeout(r, 250));

  if (calledAt != lastInput.value) return;
  selected.value = 0;

  if (!input.value.length) {
    searchResults.value = null;
    return;
  }

  const result = await commands.searchDb(input.value);
  if (result.status === "error") return console.log(result.error);

  searchResults.value = result.data;
});

/**
 * Handle the keydown event when dialog is shown.
 *
 * `Escape` is pressed, close the dialog.
 *
 * `ArrowUp` is pressed, focus element `n - 1` from searchResults (goes up).
 *
 * `ArrowDown` is pressed, focus element `n + 1` from searchResults (goes down).
 *
 * `Enter` is pressed, router goes to selected element.
 *
 * ## When input is not in focus
 *
 * `Backspace` is pressed, remove one letter from input value
 *
 * `Any` key is pressed (beside the ones above), add it to input value
 */
function handleKeyDown(e: KeyboardEvent) {
  if (e.key === "Escape") {
    showDialog.value = false;
  } else if (e.key === "ArrowUp") {
    if (!searchResults.value) return;
    selected.value = clamp(selected.value - 1);
    resultElements.value[selected.value].scrollIntoView({ behavior: "smooth" });
  } else if (e.key === "ArrowDown") {
    if (!searchResults.value) return;
    selected.value = clamp(selected.value + 1);
    resultElements.value[selected.value].scrollIntoView({ behavior: "smooth" });
  } else if (e.key === "Enter") {
    if (!searchResults.value) return;
    const result = searchResults.value[selected.value];
    updateDialog();
    router.push(`/${result.search_type}/${result.search_id}`);
  }

  // global key listening
  if (focused.value) return;
  if (e.key === "Backspace") {
    if (e.ctrlKey) {
      const lastSpace = input.value.lastIndexOf(" ");
      input.value = input.value.substring(0, lastSpace);
    } else input.value = input.value.substring(0, input.value.length - 1);
  } else if (e.key.length === 1) {
    input.value += e.key;
  }
}

/**
 * Show or hide the dialog.
 *
 * If the dialog is set to show, `nextTick()` is called, and then
 * the input element is focused.
 */
function updateDialog() {
  showDialog.value = !showDialog.value;
  if (showDialog.value) {
    nextTick(() => {
      inputElement.value.focus();
    });
  } else {
    input.value = "";
    selected.value = 0;
  }
}

useEventListener("keydown", (e) => {
  if (e.ctrlKey && e.key.toLowerCase() === "f") {
    updateDialog();
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
