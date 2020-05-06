# panopticon-tui

Terminal UI diagnostic tool.

Currently supports:
- [ZIO-ZMX](https://github.com/zio/zio-zmx)
- [Slick + HikariCP](https://scala-slick.org/doc/3.2.0/config.html#monitoring) (over JMX)

## Usage

The only way to run Panopticon currently is to build it from sources:
```
cargo build
./target/debug/panopticon-tui [OPTIONS]
```

To get a detailed help message, run:
```
panopticon-tui --help
```

Currently, Panopticon UI is using tabs. Tabs you going to see depend on what options you use to launch Panopticon (see further).

### Connecting to zio-zmx server

[ZIO-ZMX](https://github.com/zio/zio-zmx) is a tool for monitoring ZIO-based apps.
You can specify zio-zmx server address with `--zio-zmx` option. In this case Panopticon will connect to it and show a ZMX tab:
```
panopticon-tui --zio-zmx localhost:6789
```

### Database metrics over JMX

Panopticon can show database metrics, if your app exposes them via JMX. Slick and HikariCP are the only supported options at the moment.

Slick tab will be shown in Panopticon if you specify these two options:

```
panopticon-tui --jmx localhost:9010 --db-pool-name myDb
```

Here `db-pool-name` is a connection pool name, used to qualify JMX beans for Slick and/or HikariCP. 

See [this section](https://scala-slick.org/doc/3.2.0/config.html#monitoring) of Slick docs for details about setting up your app to expose db metrics over JMX.


## Build from sources

Development build:
```
cargo build
```

Optimized release build:
```
cargo build --release
```
