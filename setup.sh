#!/bin/bash

cargo build --examples

# Set CAP_NET_ADMIN for each example.
ls ./target/debug/examples/* | while read line
do
	sudo setcap "cap_net_admin=ep" $line
done
