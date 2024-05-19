import { Tracks } from "../bindings";

export function setPlayerTrack(track: Tracks) {
    localStorage.setItem("playerTrack", JSON.stringify(track));

    setPlayerProgress(0);

    window.dispatchEvent(new CustomEvent("playerTrackChanged", {
        detail: track
    }));
}

export function setCurrentPage(page: string) {
    localStorage.setItem("currentPage", page);
}

export function setPlayerVolume(volume: number) {
    localStorage.setItem("playerVolume", volume.toString());
}

export function setPlayerProgress(progress: number) {
    localStorage.setItem("playerProgress", progress.toString());
}

export function getPlayerTrack(): Tracks {
    const track = localStorage.getItem("playerTrack");
    return track ? JSON.parse(track) : {
        cover_path: "/placeholder.png",
    };
}

export function getCurrentPage(): string {
    return localStorage.getItem("currentPage") || "/";
}

export function getPlayerVolume(): number {
    return parseFloat(localStorage.getItem("playerVolume") || "0.5");
}

export function getPlayerProgress(): number {
    return parseInt(localStorage.getItem("playerProgress") || "0");
}