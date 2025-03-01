<template>
  <PaginationRoot
    :total="props.total"
    :items-per-page="props.itemsPerPage"
    show-edges
    v-model:page="page"
  >
    <PaginationList
      v-slot="{ items }"
      class="text-supporting flex items-center gap-1"
    >
      <template v-for="(page, index) in items">
        <PaginationListItem
          v-if="page.type === 'page'"
          :key="index"
          class="data-[selected]:text-text data-[selected]:bg-stroke-100 hover:bg-stroke-200 border-stroke-100 hover:text-text h-9 w-9 rounded-lg border transition data-[selected]:shadow-sm"
          :value="page.value"
        >
          {{ page.value }}
        </PaginationListItem>
        <PaginationEllipsis
          v-else
          :key="page.type"
          :index="index"
          class="flex h-9 w-9 items-center justify-center"
        >
          <!-- ellipsis character -->
          &#8230;
        </PaginationEllipsis>
      </template>
    </PaginationList>
  </PaginationRoot>
</template>

<script setup lang="ts">
import {
  PaginationEllipsis,
  PaginationList,
  PaginationListItem,
  PaginationRoot,
} from "reka-ui";

const props = defineProps<{
  total: number;
  itemsPerPage: number;
}>();

const page = defineModel<number>("page", { default: 1 });
</script>
