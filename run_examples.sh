#!/bin/bash

# Exit if any command failed.
set -e

cargo build --examples

ls ./target/debug/examples/* | grep /[a-zA-Z0-9_]*$ | while read example
do
	sudo setcap "cap_net_admin=ep" $example

	echo "Running $example..."
	$example
done

