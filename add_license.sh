#!/bin/sh

for f in **/*.rs; do
 	cat license_header $f > $f.new
	mv $f.new $f
done
