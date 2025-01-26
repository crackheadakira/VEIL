export function makeReadableTime(seconds: number) {
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.floor(seconds % 60);
  return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
}

export function makeTime(seconds: number) {
  let time = "";
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const remainingSeconds = Math.floor(seconds % 60);

  if (hours > 0) {
    time += `${hours} hour${hours > 1 ? "s" : ""} `;
  }

  if (minutes > 0) {
    time += `${minutes} min `;
  }

  time += `${remainingSeconds} sec`;

  return time;
}

import { convertFileSrc } from "@tauri-apps/api/core";
import { exists } from "@tauri-apps/plugin-fs";

export async function placeholderIfEmpty(imagePath: string | undefined) {
  if (!imagePath) return "/placeholder.png";
  const imageExists = await exists(imagePath);
  return imageExists ? convertFileSrc(imagePath) : "/placeholder.png";
}

export function resetStore() {
  localStorage.clear();
  window.location.reload();
}
