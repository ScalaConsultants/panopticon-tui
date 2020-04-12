# panopticon

Terminal UI diagnostic tool for your applications.

Currently supports:
- ZIO
- Zookeeper

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

Panopticon uses `netcat` to communicate with Zookeeper nodes, so it requires `nc` to be available in the terminal.

To run it you need to pass a list of addresses for Zookeeper nodes like this:
```
./panopticon my_host_x.com:2181 my_host_y.com:2181 my_host_z.com:2181
```
