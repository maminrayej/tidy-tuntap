`tidy-tuntap` is a Rust wrapper for working with TUN/TAP devices in Linux.

## Examples
There are examples of using this crate in the `examples` folder which also act as tests. To run these examples you need to build them and set the `cap_net_admin` capability of each generated executable. There is a convenience shell script in the root folder of the project called `run_examples.sh` which you can execute to build and run all the examples. Feel free to inspect the contents of this script and all the examples before trying to running them.

## Features
* Multiqueue support
* Async support
* IPv6 support

## Roadmap
- [ ] Windows support
- [ ] MacOS support
- [ ] OpenBSD support
