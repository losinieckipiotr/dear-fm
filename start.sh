#!/bin/bash

cargo run 2>&1 | tee app.log
# cargo run --release
