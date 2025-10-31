# Sodapop Reimagined

## High Priority

- [x] Database
  - [x] Database pool
  - [x] Lessening repetiton in db.rs
  - [x] ~~Add support for featuring artists (e.g Tyler The Creator, Daniel Caesar)~~
- [x] Migrate away from audiotags and use custom readers
- [ ] ID3
  - [x] Find a way to calculate duration for ID3 files (we just have the user play the track to update it)
  - [ ] Add support for ID3v2.4
- [x] Better error handling (backend)
  - [x] FLAC
  - [x] ID3
  - [x] Database
  - [x] Commands
- [ ] Better queue system
- [ ] Logging System
  - [ ] Log warn / error instead of unwrapping
- [ ] Add a fullscreen view
- [x] Add support for a custom CSS folder

## Medium Priority

- [x] Playlist support
  - [x] Database methods
  - [x] Context menu
  - [x] Playlist page
- [x] Add last.fm support
  - [x] Allow adding session key
  - [x] Scrobble track > 30s
  - [ ] Get metadata for missing files
- [ ] Liked Songs
- [ ] Artist page
  - [ ] Allow user to view their liked songs for given artist
  - [ ] Play random track from artist
- [x] Settings page
- [ ] Enabling Discord RPC mid-track without changing anything doesn't send RPC
  - [ ] Then when pausing & resuming gives a basic "Browsing" status

## Low Priority

- [ ] Add documentation / comments
  - [x] Frontend
  - [ ] Backend
- [x] Implement souvlaki
  - [x] Get it working on backend side
  - [x] Control frontend actions
  - [ ] Desync when seeking
- [ ] Lyric support
  - [ ] Read .LRC files
  - [ ] Lyric View
- [x] Discord RPC
