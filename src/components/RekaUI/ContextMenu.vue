<template>
  <ContextMenuRoot v-model:open="open">
    <ContextMenuTrigger as-child>
      <slot></slot>
    </ContextMenuTrigger>

    <ContextMenuPortal v-if="open">
      <ContextMenuContent
        class="border-border-secondary data-[side=top]:animate-slideDownAndFade data-[side=right]:animate-slideLeftAndFade data-[side=bottom]:animate-slideUpAndFade data-[side=left]:animate-slideRightAndFade bg-bg-secondary z-30 w-fit rounded-md border p-1 will-change-[opacity,transform]"
        :side-offset="5"
      >
        <ContextMenuSub>
          <ContextMenuSubTrigger
            v-model:open="subOpen"
            class="group context-menu-item w-full pr-0"
          >
            <span class="i-fluent-add-24-regular"></span>
            <small>Add to Playlist</small>
            <div class="pl-5">
              <span
                class="i-fluent-chevron-right-24-regular ml-auto text-xs"
              ></span>
            </div>
          </ContextMenuSubTrigger>

          <ContextMenuPortal v-if="subOpen">
            <ContextMenuSubContent
              class="border-border-secondary data-[side=top]:animate-slideDownAndFade data-[side=right]:animate-slideLeftAndFade data-[side=bottom]:animate-slideUpAndFade data-[side=left]:animate-slideRightAndFade bg-bg-secondary z-30 w-fit rounded-md border p-1 will-change-[opacity,transform]"
              :side-offset="2"
              :align-offset="-5"
            >
              <ContextMenuItem
                class="group context-menu-item"
                @select="showDialog = true"
              >
                <span class="i-fluent-star-add-24-regular"></span>
                <small>Create new Playlist</small>
              </ContextMenuItem>

              <ContextMenuItem
                v-for="playlist of props.playlists"
                class="group context-menu-item"
                @select="$emit('playlist', 'add', playlist.id, props.track.id)"
              >
                <small>{{ playlist.name }}</small>
              </ContextMenuItem>
            </ContextMenuSubContent>
          </ContextMenuPortal>
        </ContextMenuSub>

        <ContextMenuItem
          v-if="playlist_id"
          class="group context-menu-item"
          @select="$emit('playlist', 'remove', playlist_id, props.track.id)"
        >
          <span class="i-fluent-delete-24-regular"></span>
          <small>Remove from Playlist</small>
        </ContextMenuItem>

        <ContextMenuItem
          class="group context-menu-item"
          @select="addToPersonalQueue"
        >
          <span class="i-fluent-add-square-multiple-24-regular"></span>
          <small>Add to Queue</small>
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenuPortal>
  </ContextMenuRoot>

  <Modal hide-trigger v-model="showDialog">
    <Dialog
      title="New Playlist"
      placeholder="New Playlist"
      @cancel="showDialog = false"
      @submit="
        (name: string) => (
          $emit('create-playlist', name, props.track.id),
          (showDialog = false)
        )
      "
    />
  </Modal>
</template>

<script setup lang="ts">
import { events, Playlists, Tracks } from "@/composables/";
import { Dialog, Modal } from "@/components/";
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
import { ref } from "vue";

const showDialog = ref(false);
const open = ref(false);
const subOpen = ref(false);

const props = defineProps<{
  playlist_id?: number;
  track: Tracks;
  playlists: Playlists[] | null;
}>();

defineEmits<{
  (
    e: "playlist",
    type: "add" | "remove",
    playlistId: number,
    trackId: number,
  ): void;
  (e: "create-playlist", name: string, trackId: number): void;
}>();

async function addToPersonalQueue() {
  await events.queueEvent.emit({
    type: "EnqueuePersonal",
    data: { track_id: props.track.id },
  });
}
</script>
