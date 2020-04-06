#!/bin/bash

FOLDER=$(dirname "$0")
cd $FOLDER
export RUST_LOG=info
nohup ../target/release/ipserver 8080 &