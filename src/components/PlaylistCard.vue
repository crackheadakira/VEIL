<template>
  <div
    class="aspect-card veil-card hover:border-border-secondary-hovered group flex h-28 cursor-pointer items-center gap-6 p-4 duration-150"
  >
    <img
      class="aspect-square w-20 rounded-md duration-150 group-hover:opacity-90"
      src="/placeholder.png"
      alt="Album Cover"
    />
    <div class="flex w-72 flex-col gap-1 select-none">
      <p class="text-text-primary truncate">
        Nice long playlist name that should cut off as well
      </p>

      <small class="text-text-secondary block truncate">
        This is a really long playlist description that should cut off
      </small>
    </div>
  </div>
</template>
