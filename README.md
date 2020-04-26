# panopticon-tui

Terminal UI diagnostic tool.

Currently supports:
- ZIO-ZMX

## build

for dev build
```
cargo build
```

for optimized release build
```
cargo build --release
```

## run

To run it you need to pass an addresses of ZIO-ZMX server:
```
./target/debug/panopticon-tui -zio-zmx localhost:1111
```
