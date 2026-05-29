# NES Emulator in Rust

[![stability-experimental](https://img.shields.io/badge/stability-experimental-orange.svg)](https://github.com/mkenney/software-guides/blob/master/STABILITY-BADGES.md#experimental)
[![Open Source Saturday Italy](https://img.shields.io/badge/Open%20Source%20Saturday-Italy-red)](https://oss-italy.github.io/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[![Crafted by Human](https://madebyhuman.iamjarl.com/badges/crafted-white.svg)](https://madebyhuman.iamjarl.com)


This is my version of the NES emulator written in Rust.

## Progress

| module  | progress |
|---------|----------|
| CPU     | 85%      |
| BUS     | 80%      |
| ROM     | 100%     |
| PPU     | 0%       |
| GamePad | 0%       |
| APU     | 0%       |

## Dev env setup

Install SDL2 library and configure Rust bindings with this [simple guide](https://github.com/Rust-SDL2/rust-sdl2).

## Git hooks

```shell
git config core.hooksPath .githooks
```

## Testing ROMs

### Snake test ROM

```shell
cargo run snake_test
```

Runs `cargo fmt --check` and `cargo test` on every commit.

### Nestest ROM
```shell
DEBUG=1 cargo run nestest
```