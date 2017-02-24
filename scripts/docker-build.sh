#!/bin/bash
docker build --no-cache=true --tag ethcore/parity:$1 .
docker push ethcore/parity:$1
