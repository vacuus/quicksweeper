**Quicksweeper**

Quicksweeper is a multiplayer game that seeks to redefine the Minesweeper genre by providing many original game modes. Quicksweeper is also being designed to allow pluggability for game modes, to enable endless fun (within the bounds of human creativity)!

Quicksweeper is currently in early development. Networking is a priority.

### Web

WASM is being experimentally supported. Install `wasm-server-runner` to try quicksweeper as a web
app:

```sh
$ cargo install wasm-server-runner
```

Currently this crashes and does nothing if loading of `Field` is attempted. 