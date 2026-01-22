<template>
  <div class="bg-bg-primary flex flex-col gap-4">
    <h4 class="text-text-primary">Text</h4>

    <div class="flex flex-col gap-4">
      <h6 class="text-text-primary">text-primary</h6>
      <div class="text-text-primary flex w-full gap-8">
        <p
          class="active:text-text-primary-active hover:text-text-primary-hovered cursor-pointer"
        >
          Base color.
        </p>
        <p class="text-text-primary-disabled cursor-not-allowed">Disabled.</p>
      </div>

      <h6 class="text-text-secondary">text-secondary</h6>
      <div class="text-text-secondary flex w-full gap-8">
        <p
          class="active:text-text-secondary-active hover:text-text-secondary-hovered cursor-pointer"
        >
          Base color.
        </p>
        <p class="text-text-secondary-disabled cursor-not-allowed">Disabled.</p>
      </div>

      <h6 class="text-text-tertiary">text-tertiary</h6>
      <div class="text-text-tertiary flex w-full gap-8">
        <p
          class="active:text-text-tertiary-active hover:text-text-tertiary-hovered cursor-pointer"
        >
          Base color.
        </p>
        <p class="text-text-tertiary-disabled cursor-not-allowed">Disabled.</p>
      </div>

      <h6 class="text-text-primary">Pairings</h6>
      <div class="flex w-full gap-8">
        <div class="flex flex-col gap-2">
          <p class="text-text-primary">Base color.</p>
          <p class="text-text-secondary">Base color.</p>
          <p class="text-text-tertiary">Base color.</p>
        </div>

        <div class="flex flex-col gap-2 *:cursor-not-allowed">
          <p class="text-text-primary-disabled">Disabled.</p>
          <p class="text-text-secondary-disabled">Disabled.</p>
          <p class="text-text-tertiary-disabled">Disabled.</p>
        </div>

        <div class="flex flex-col gap-2">
          <p class="text-text-primary">Base color.</p>
          <small class="text-text-secondary">Base color.</small>
          <small class="text-text-tertiary">Base color.</small>
        </div>

        <div class="flex flex-col gap-2">
          <h6 class="text-text-primary">Base color.</h6>
          <p class="text-text-secondary">Base color.</p>
          <small class="text-text-tertiary">Base color.</small>
        </div>
      </div>
    </div>

    <h4 class="text-text-primary">Color</h4>
    <div class="flex flex-col gap-4">
      <h6 class="text-text-primary">bg-primary</h6>
      <div class="flex w-full gap-8">
        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-primary</p>
          <div
            class="bg-bg-primary border-border-primary hover:border-border-primary-hovered active:border-border-primary-active size-16 border"
          />
        </div>

        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-secondary</p>
          <div
            class="bg-bg-primary border-border-secondary hover:border-border-secondary-hovered active:border-border-secondary-active size-16 border"
          />
        </div>

        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-primary + secondary</p>
          <div
            class="bg-bg-primary border-border-primary flex size-16 items-center justify-center border"
          >
            <div
              class="bg-bg-secondary border-border-secondary hover:border-border-secondary-hovered active:border-border-secondary-active size-8 border"
            />
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-primary-disabled</p>
          <div
            class="bg-bg-primary border-border-primary-disabled size-16 cursor-not-allowed border"
          />
        </div>

        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-secondary-disabled</p>
          <div
            class="bg-bg-primary border-border-secondary-disabled size-16 cursor-not-allowed border"
          />
        </div>
      </div>

      <h6 class="text-text-primary">bg-secondary</h6>
      <div class="flex w-full gap-8">
        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-primary</p>
          <div
            class="bg-bg-secondary border-border-primary hover:border-border-primary-hovered active:border-border-primary-active size-16 border"
          />
        </div>

        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-secondary</p>
          <div
            class="bg-bg-secondary border-border-secondary hover:border-border-secondary-hovered active:border-border-secondary-active size-16 border"
          />
        </div>

        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-primary + secondary</p>
          <div
            class="bg-bg-secondary border-border-primary flex size-16 items-center justify-center border"
          >
            <div
              class="bg-bg-secondary border-border-secondary hover:border-border-secondary-hovered active:border-border-secondary-active size-8 border"
            />
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-primary-disabled</p>
          <div
            class="bg-bg-secondary border-border-primary-disabled size-16 cursor-not-allowed border"
          />
        </div>

        <div class="flex flex-col gap-2">
          <p class="text-text-primary">border-secondary-disabled</p>
          <div
            class="bg-bg-secondary border-border-secondary-disabled size-16 cursor-not-allowed border"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useConfigStore } from "@/composables/";
import { onMounted } from "vue";

const configStore = useConfigStore();
onMounted(() => {
  configStore.currentPage = "/color_testing";
  configStore.pageName = "Color Testing";
});
</script>
