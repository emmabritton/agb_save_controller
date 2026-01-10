# ???

A gba game?

## Screenshots

## Player Usage

Download gba file from [here](https://github.com/???/???/releases/latest) and run in an emulator (mGBA recommended)

### Dev Usage

First follow instructions at https://agbrs.dev/book/setup/getting_started.html

#### Run

`cargo run`

(runs in mGBA)

#### Test

`cargo test --package game`
`cargo test --package resources`

(runs in mGBA)

#### Make gba file

`agb-gbafix target/thumbv4t-none-eabi/release/??? -o ???.gba`

## Thanks/Tools

- agb
  - https://agbrs.dev/
  - Framework for running rust on GBA
- mGBA
  - https://mgba.io/
  - Testing
- aseprite
  - https://www.aseprite.org/
  - Creating backgrounds and sprites