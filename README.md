# microbit-musicbox-rs

This music player supports various operations with the buttons and the accelerometer on the microbit v2 development board.

- Button A
  - Single click: Decrease the volume by one level
  - Double click: Play the previous song
  - Long press: Decrease the volume continuously until released
- Button B
  - Single click: Increase the volume by one level
  - Double click: Play the next song
  - Long press: Increase the volume continuously until released
- Shake
  - Play or pause the music

## Prerequisites

### Hardware

* [BBC Micro:bit v2](https://microbit.org/new-microbit/)

### Software

Make sure you have the latest versions (`cargo install <tool>`) of these tools:

* `flip-link`
* `probe-rs-tools`

## Usage

### run application

```
cargo run
```

### flush application

```
cargo embed
```

## License

This project is licensed under the MIT license, see [MIT license](LICENSE) file for details.
