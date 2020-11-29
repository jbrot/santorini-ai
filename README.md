# Santorini AI

This program provides a terminal interface for the board game [Santorini](https://boardgamegeek.com/boardgame/194655/santorini), along with a competent computer opponent.

My goals for this project are:
1. to refine my understanding of implementing a game AI from what I learned with [jstris-ai](https://github.com/jbrot/jstris-ai), and
2. to learn Rust.

# Written AIs

## Random AI

This AI moves randomly.

## Heuristic AI

This AI looks a few moves in the future with the following goals:
1. Win
2. Prevent the opponent from winning
3. Both:
   - Minimize distance between the AI pawns and the opponents' pawns
   - Maximize the height of the AI pawns relative to the opponents' pawns, where a pawn's height is computed by looking at its height and the height of its neighbors.

# Status

- [x] Create core game datatypes
- [x] Create visual front end for game
- [x] Expand TUI to allow for inputting moves
- [x] Implement rules
- [x] Create a heuristic-based AI
- [ ] Create a MCTS AI
- [ ] Stretch: Create and train an Alpha Zero AI

Currently, this repo contains a library that encapsulates the gameplay of Santorini and a TUI front end using [tui-rs](https://github.com/fdehau/tui-rs) and [termion](https://github.com/redox-os/termion) that allows for a two-player game to be played.
The TUI is minimal but fully functional, and the underlying game library is extensively unit tested.
The TUI allows for a two player game and a one player game against the heuristic AI.

Up next, I plan on allowing each player to be assigned to one of the AIs or a human in the TUI.

# License

This project is licensed under the GPLv3.
