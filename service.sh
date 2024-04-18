#!/bin/sh

cargo build -r
rm ~/.local/bin/hid-io-ergoone
cp ./target/release/hid-io-ergoone ~/.local/bin
hyprctl reload
