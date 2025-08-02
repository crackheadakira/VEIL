<template>
  <ContextMenuRoot>
    <ContextMenuTrigger as-child>
      <slot></slot>
    </ContextMenuTrigger>
    <ContextMenuPortal>
      <ContextMenuContent
        class="border-border-secondary data-[side=top]:animate-slideDownAndFade data-[side=right]:animate-slideLeftAndFade data-[side=bottom]:animate-slideUpAndFade data-[side=left]:animate-slideRightAndFade bg-bg-secondary z-30 w-fit rounded-md border p-1 will-change-[opacity,transform]"
        :side-offset="5"
      >
        <ContextMenuSub v-if="props.playlists && props.playlists.length">
          <ContextMenuSubTrigger class="group context-menu-item w-full pr-0">
            <span class="i-fluent-add-24-regular"></span>
            <small>Add to Playlist</small>
            <div class="pl-5">
              <span
                class="i-fluent-chevron-right-24-regular ml-auto text-xs"
              ></span>
            </div>
          </ContextMenuSubTrigger>
          <ContextMenuPortal>
            <ContextMenuSubContent
              class="border-border-secondary data-[side=top]:animate-slideDownAndFade data-[side=right]:animate-slideLeftAndFade data-[side=bottom]:animate-slideUpAndFade data-[side=left]:animate-slideRightAndFade bg-bg-secondary z-30 w-fit rounded-md border p-1 will-change-[opacity,transform]"
              :side-offset="2"
              :align-offset="-5"
            >
              <ContextMenuItem
                v-for="playlist of props.playlists"
                class="group context-menu-item"
                @select="$emit('playlist', 'add', playlist, props.track)"
              >
                <small>{{ playlist.name }}</small>
              </ContextMenuItem>
            </ContextMenuSubContent>
          </ContextMenuPortal>
        </ContextMenuSub>

        <ContextMenuItem
          v-if="curr_playlist"
          class="group context-menu-item"
          @select="$emit('playlist', 'remove', curr_playlist, props.track)"
        >
          <span class="i-fluent-delete-24-regular"></span>
          <small>Remove from Playlist</small>
        </ContextMenuItem>

        <ContextMenuItem
          class="group context-menu-item"
          @select="$emit('queue', props.track)"
        >
          <span class="i-fluent-add-square-multiple-24-regular"></span>
          <small>Add to Queue</small>
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenuPortal>
  </ContextMenuRoot>
</template>

<script setup lang="ts">
import { Playlists, Tracks } from "@/composables/";
import {
  ContextMenuRoot,
  ContextMenuTrigger,
  ContextMenuPortal,
  ContextMenuItem,
  ContextMenuContent,
  ContextMenuSub,
  ContextMenuSubTrigger,
  ContextMenuSubContent,
} from "reka-ui";

const props = defineProps<{
  curr_playlist?: Playlists;
  track: Tracks;
  playlists: Playlists[] | null;
}>();

defineEmits<{
  (e: "queue", payload: Tracks): void;
  (
    e: "playlist",
    type: "add" | "remove",
    playlist: Playlists,
    track: Tracks,
  ): void;
}>();
</script>
