# Sodapop Reimagined

## High Priority

- [x] Database
  - [x] Database pool
  - [x] Lessening repetiton in db.rs
  - [x] Add support for featuring artists (e.g Tyler The Creator, Daniel Caesar)
- [x] Migrate away from audiotags and use custom readers
- [ ] ID3
  - [x] Find a way to calculate duration for ID3 files (we just have the user play the track to update it)
  - [ ] Add support for ID3v2.4
- [x] Better error handling (backend)
  - [x] Metadata
    - [x] FLAC
    - [x] ID3
  - [x] Database
  - [x] Commands

## Medium Priority

- [ ] Playlist support
  - [x] Database methods
  - [ ] Context menu
  - [ ] Playlist page
- [ ] Add last.fm support
  - [ ] Allow adding custom key
  - [ ] Scrobble track > 30s
- [ ] Artist page

## Low Priority

- [ ] Add documentation / comments
  - [ ] Frontend
  - [ ] Backend
- [x] Implement souvlaki
  - [x] Get it working on backend side
  - [x] Control frontend actions
  - [ ] Desync when seeking
- [ ] Lyric support
