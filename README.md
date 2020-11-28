# Santorini AI

This program aims to provide a terminal interface for the board game [Santorini](https://boardgamegeek.com/boardgame/194655/santorini), along with a competent computer opponent.

My goals for this project are:
1. to refine my understanding of implementing a game AI from what I learned with [jstris-ai](https://github.com/jbrot/jstris-ai), and
2. to learn Rust.

# Status

- [x] Create core game datatypes
- [x] Create visual front end for game
- [x] Expand TUI to allow for inputting moves
- [x] Implement rules
- [ ] Create a simple heuristic-based AI
- [ ] Create a MCTS AI
- [ ] Stretch: Create and train an Alpha Zero AI

Currently, this repo contains a library that encapsulates the gameplay of Santorini and a TUI front end using [tui-rs](https://github.com/fdehau/tui-rs) and [termion](https://github.com/redox-os/termion) that allows for a two-player game to be played.
The TUI is minimal but fully functional, and the underlying game library is extensively unit tested.

Up next, I plan on implementing a trivial AI and adding a menu to the TUI to allow the user to select the game type.

# License

This project is licensed under the GPLv3.
