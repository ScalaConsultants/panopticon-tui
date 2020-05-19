# panopticon-tui

![CI](https://github.com/ScalaConsultants/panopticon-tui/workflows/Rust%20CI/badge.svg)

Terminal UI diagnostic tool.

Currently supports:
- [ZIO-ZMX](https://github.com/zio/zio-zmx)
- [Slick + HikariCP](https://scala-slick.org/doc/3.2.0/config.html#monitoring) (over JMX)
- Akka actor metrics (via [akka-actor-tree](https://github.com/ScalaConsultants/akka-actor-tree))

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

[ZIO-ZMX](https://github.com/zio/zio-zmx) is a tool for monitoring ZIO-based apps. With panopticon you can see the fiber tree visualized and monitor their number:

![ZIO tab demo](./assets/zio-demo.png)

You can specify zio-zmx server address with `--zio-zmx` option. In this case Panopticon will connect to it and show a ZMX tab:
```
panopticon-tui --zio-zmx localhost:6789
```

**⚠️ WARNING**: Currently, zio-zmx doesn't provide efficient ways of getting fiber count metrics, so Panopticon has to do a full fiber dump each tick to calculate them. Make sure your `tick-rate` isn't too frequent.

### Database metrics over JMX

Panopticon can show database metrics, if your app exposes them via JMX. Slick and HikariCP are the only supported options at the moment.

![Slick tab demo](./assets/slick-demo.png)

Slick tab will be shown in Panopticon if you specify these two options:

```
panopticon-tui --jmx localhost:9010 --db-pool-name myDb
```

Here `db-pool-name` is a connection pool name, used to qualify JMX beans for Slick and/or HikariCP. 

See [this section](https://scala-slick.org/doc/3.2.0/config.html#monitoring) of Slick docs for details about setting up your app to expose db metrics over JMX.


### Akka metrics

Panopticon can also display an entire tree of actors under some actor system. As well as monitor total amount of actors in time.

![Slick tab demo](./assets/akka-demo.png)

To use this feature, however, you'd have to enable publication of this data in your application. There's the [akka-actor-tree](https://github.com/ScalaConsultants/akka-actor-tree) library, specifically suited for that purpose. Checkout it's README for the detauls on how you can set it up.

Only HTTP way of transfer is supported for at the moment. To use it and see the actor data on a separate tab, launch Panopticon with following options:

```
panopticon-tui --actor-tree http://localhost:8080/actor-tree --actor-count http://localhost:8080/actor-count
```

Replace the endpoint urls with the ones you set up with [akka-actor-tree](https://github.com/ScalaConsultants/akka-actor-tree).

## Build from sources

Development build:
```
cargo build
```

Optimized release build:
```
cargo build --release
```
