# VEIL

A local music player built with **Rust**, **Vue.js**, with **Tauri**. It integrates with **Last.fm** for scrobbling and **Discord RPC** to show what you're listening to.

## Features

- **Local Music Playback**: Play your local music files with an intuitive interface built using Vue.js.
- **Last.fm Scrobbling**: Automatically scrobble tracks to your Last.fm account.
- **Discord Rich Presence**: Show what you're listening to on Discord with Discord RPC support.

## Requirements

- [Rust](https://www.rust-lang.org/)
- [Bun](https://bun.sh/) (for building the frontend)

## Optional

- A **Last.fm** account for scrobbling.
- **Discord** installed for Rich Presence.

###### Last.FM & Discord are opt-in

## Installation

### 1. Clone the Repository

```bash
git clone https://github.com/crackheadakira/VEIL.git
cd VEIL
```

### 2. Install frontend dependencies

```bash
bun i
```

### 3. Run the application locally

```bash
bun tauri dev
```

### 4. Build the application

```bash
bun tauri build
```

# License

This project is licensed under the Apache 2.0 license - see the [LICENSE](LICENSE) file for details.
