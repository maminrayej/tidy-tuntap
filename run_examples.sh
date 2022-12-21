#!/bin/bash

# Exit if any command failed.
set -e

cargo build --examples --all-features

# Cargo builds each example and places it under the examples folder.
# If the example is called `my_example.rs`, the generated executable will be named `my_example`.
# But, the examples folder contains other files that start with `my_example` but contain a random string afterwards.
# So for example if we have:
# 	examples/example1.rs 
#	examples/example2.rs
# We will have:
#	./target/debug/examples/example1
#	./target/debug/examples/example1-random-string
#	./target/debug/examples/example1-random-string
#	./target/debug/examples/example2
#	./target/debug/examples/example2-random-string
#	./target/debug/examples/example2-random-string
# We only want to execute the ones that don't have a random string in their name.
# So we filter the executable files in the examples directory using grep to not list
# the files that contain a random string. Luckily, these files all have `-` in their name.
find target/debug/examples/ -regex ".*/[0-9a-zA-Z_]+$" | while read -r example
do
	# In order for the generated executable to be able to create a TUN/TAP device,
	# and perform other functionalities related to network administration, 
	# it must have the CAP_NET_ADMIN capability. So we set the CAP_NET_ADMIN in both
	# its Effective and Permitted sets, hence the "cap_net_admin=ep".
	# For more info read:
	#	* man capabilities
	#	* man setcap
	#	* man cap_from_text
	sudo setcap "cap_net_admin=ep" "$example"

	echo "Running $(basename -- "$example")..."
	$example
done

