generate-tapes:
	#!/usr/bin/env bash
	for name in $(basename -s.rs examples/*.rs); do
		if [ "$name" != boilerplate ]; then
			cargo build --example $name
			just generate-tape $name examples/${name}.png
		fi
	done
	rm out.gif

generate-tape NAME OUTPUT:
	#!/usr/bin/env vhs
	Set Shell "bash"
	Set FontSize 10
	Set Width 1920
	Set Height 1680

	Hide
	Type @1ms `cargo run --example {{NAME}}`
	Enter
	Sleep 1s
	Show
	Screenshot {{OUTPUT}}
	Sleep 1s
