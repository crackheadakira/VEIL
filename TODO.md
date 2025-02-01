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

## Medium Priority

- [x] Playlist support
  - [x] Database methods
  - [x] Context menu
  - [x] Playlist page
- [ ] Add last.fm support
  - [ ] Allow adding custom key
  - [ ] Scrobble track > 30s
  - [ ] Get metadata for missing files
- [ ] Artist page
- [ ] Settings page

## Low Priority

- [ ] Add documentation / comments
  - [x] Frontend
  - [ ] Backend
- [x] Implement souvlaki
  - [x] Get it working on backend side
  - [x] Control frontend actions
  - [ ] Desync when seeking
- [ ] Lyric support
- [ ] Discord RPC
