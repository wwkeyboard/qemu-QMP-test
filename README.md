# qemu-QMP-test


# Running

## Starting QEMU

On a mac you can easily get an instance running on QEMU with a socket open using [LIMA](https://github.com/lima-vm/lima). `brew install lima` then `limactl start` and you're set. 

I don't know of a quick way to achieve something similar on Linux, but the [Arch Wiki](https://wiki.archlinux.org/title/QEMU) has some great docs that should help.

## Running the listener

The default method of running only requires a path to the socket, for example in development it would be:

```
    $ RUST_LOG=trace cargo run -- -p ~/.lima/default/qmp.sock
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
