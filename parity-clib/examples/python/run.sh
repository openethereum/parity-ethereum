#!/usr/bin/env bash

mkdir -p ./venv
python3 -m venv ./venv

source ./venv/bin/activate

pushd ../../python
pip install -r requirements.txt
python setup.py install
popd

python example.py
