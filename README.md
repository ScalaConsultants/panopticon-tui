# warden

## build

for debug
```
cargo build
```

for release
```
cargo build --release
```

## run

Warden uses `netcat` to communicate with Zookeeper nodes, so it requires `nc` to be available in terminal.

To run it you need to pass a list of addresses for Zookeeper nodes like this:
```
./warden my_host_x.com:2181 my_host_y.com:2181 my_host_z.com:2181
```
