import { type FrontendError, toastBus } from "@/composables/";
import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * Returns a human-readable time string from seconds in the format `mm:ss`
 * @param {number} seconds - Time in seconds
 * @returns {string} Human-readable time string
 *
 * @example
 * // Returns "1:05"
 * makeReadableTime(65)
 *
 * @example
 * // Returns "0:12"
 * makeReadableTime(12)
 */
export function formatTime(format: "mm:ss", seconds: number): string;
/**
 * Returns a human-readable time string from seconds in the format `x hours x mins x secs`
 * @param {number} seconds - Time in seconds
 * @returns {string} Human-readable time string
 *
 * @example
 * // Returns "1 hour 0 min 12 sec"
 * makeTime(3612)
 *
 * @example
 * // Returns "0 min 12 sec"
 * makeTime(12)
 */
export function formatTime(format: "hh:mm:ss", seconds: number): string;
export function formatTime(
  format: "mm:ss" | "hh:mm:ss",
  seconds: number,
): string {
  if (format === "mm:ss") {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes}:${remainingSeconds.toString().padStart(2, "0")}`;
  } else {
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
}

/**
 * Returns the image path if it is not empty, otherwise returns the placeholder image path.
 * @param {string} imagePath - The image path to check
 * @returns {string} The image path run through `convertFileSrc()`, or the placeholder image path
 *
 * @example
 * // Returns "asset://path/to/image.png"
 * placeholderIfEmpty("/path/to/image.png")
 *
 * @example
 * // Returns "/placeholder.png"
 * placeholderIfEmpty("")
 *
 * @example
 * // Returns "/placeholder.png"
 * placeholderIfEmpty("/placeholder.png")
 */
export function placeholderIfEmpty(imagePath: string | undefined): string {
  if (!imagePath || imagePath === "/placeholder.png") return "/placeholder.png";

  return convertFileSrc(imagePath);
}

/**
 * Handles errors by displaying a toast with the error message
 *
 * @param result - The result of a backend function
 *
 * @example
 * const result = await commands.newPlaylist("My Playlist");
 * if(result.status === "error") handleBackendError(result.error);
 */
export function handleBackendError(error: FrontendError): void {
  toastBus.addToast("error", `[${error.type}] ${error.data}`);
}
