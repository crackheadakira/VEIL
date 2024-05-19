import { Albums, Tracks } from "../bindings";

export function setPlayerTrack(track: Tracks) {
    localStorage.setItem("playerTrack", JSON.stringify(track));

    setPlayerProgress(0);

    window.dispatchEvent(new CustomEvent("playerTrackChanged", {
        detail: track
    }));
}

export function setQueue(queue: Tracks[]) {
    localStorage.setItem("queue", JSON.stringify(queue));
    const isShuffled = localStorage.getItem("shuffled") === "true";

    if (isShuffled) {
        localStorage.setItem("shuffled", "false");
        shuffleQueue();
    }
}

export function setPersonalQueue(queue: Tracks[]) {
    localStorage.setItem("personalQueue", JSON.stringify(queue));
}

export function setQueueIndex(index: number) {
    localStorage.setItem("queueIndex", index.toString());
}

export function addToPersonalQueue(track: Tracks) {
    const queue = getPersonalQueue();
    queue.push(track);
    setPersonalQueue(queue);
}

export function setRecentlyPlayed(album: Albums) {
    const albums = getRecentlyPlayed();
    const index = albums.findIndex((a) => a.id === album.id);

    if (index !== -1) albums.splice(index, 1);

    albums.unshift(album);

    if (albums.length > 10) albums.pop();

    localStorage.setItem("recentlyPlayed", JSON.stringify(albums));
}

export function getQueueIndex(): number {
    return parseInt(localStorage.getItem("queueIndex") || "0");
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

export function skipTrack(forward: boolean) {
    const queue = getQueue();
    const personalQueue = getPersonalQueue();
    const index = getQueueIndex();

    if (personalQueue.length > 0) {
        setPlayerTrack(personalQueue[0]);
        setPersonalQueue(personalQueue.slice(1));
        setQueueIndex(index);
        return;
    }

    if (forward) {
        if (index === queue.length - 1) return;
        setQueueIndex(index + 1);
        setPlayerTrack(queue[index + 1]);
    } else {
        if (index === 0) return;
        setQueueIndex(index - 1)
        setPlayerTrack(queue[index - 1]);
    }
}

export function getPersonalQueue(): Tracks[] {
    const queue = localStorage.getItem("personalQueue");
    return queue ? JSON.parse(queue) : [];
}

export function getLoop(): "none" | "track" | "queue" {
    return localStorage.getItem("loop") as "none" | "track" | "queue";
}

export function setLoop(loop: "none" | "track" | "queue") {
    localStorage.setItem("loop", loop);
}

export function loopQueue() {
    const loop = localStorage.getItem("loop");

    if (loop === "none") {
        setLoop("queue");
        return;
    }

    if (loop === "queue") {
        setLoop("track");
        return;
    }

    setLoop("none");
}

export function shuffleQueue() {
    const shuffled = localStorage.getItem("shuffled");

    if (shuffled === "true") {
        localStorage.setItem("shuffled", "false");
        setQueue(getQueue().sort((a, b) => a.id - b.id));
        return;
    }

    const queue = getQueue();
    const track = getPlayerTrack();
    const index = queue.findIndex((t) => t.id === track.id);

    if (index === -1) return;

    queue.splice(index, 1);
    queue.sort(() => Math.random() - 0.5);
    queue.unshift(track);

    setQueue(queue);
    localStorage.setItem("shuffled", "true");
}

export function getQueue(): Tracks[] {
    const queue = localStorage.getItem("queue");
    return queue ? JSON.parse(queue) : [];
}

export function getRecentlyPlayed(): Albums[] {
    const tracks = localStorage.getItem("recentlyPlayed");
    return tracks ? JSON.parse(tracks) : [];
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

export function isShuffled() {
    return localStorage.getItem("shuffled") === "true";
}