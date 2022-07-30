`tidy-tuntap` is a Rust wrapper for working with TUN/TAP devices in Linux.
For more info: [tuntap.txt](https://www.kernel.org/doc/Documentation/networking/tuntap.txt).

## Run the examples
There are examples of using this crate in the `examples` folder which also act as tests. To run these examples you need to build them and set the `cap_net_admin` capability of each generated executable. There is a convenience shell script in the root folder of the project called `run_examples.sh` which you can execute to build and run all the examples. Feel free to inspect the contents of this script and all the examples before trying to running them.

## TODO
* Figure out how to set the IPv6 addresses for the device and its destination.
* Figure out why the ioctl fails when setting the `metric` of the device.
* Provide an async API for tokio and expose it behind a feature gate.
* Provide support for `multi-queue`.