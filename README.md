# qemu-QMP-test


# Running

The default method of running only requires a path to the socket, for example in development it would be:

```
    $ cargo run -- ~/.lima/default/qmp.sock
```

or if you've built it in release:

```
   $ qemu-qmp-test ~/.lima/default/qmp.sock
```

## Debugging

Setting the `RUST_LOG` environment variable to `trace` will dump transmission information to stdout:

```
RUST_LOG=trace cargo run -- ~/.lima/default/qmp.sock
```
