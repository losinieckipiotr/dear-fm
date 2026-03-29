#!/bin/bash

cargo run --release 2>&1 | tee app.log
# cargo run --release
