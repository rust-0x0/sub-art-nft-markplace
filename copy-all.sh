#!/usr/bin/env bash

# set -eu
basepath=$(pwd)
srcpath=basepath/wyvern-ink-contracts-substrate/
destpath=../abi/
addrs=(five_degrees erc1155)
for i in 1 2; do cp -f ${srcpath}${addrs[i-1]}/target/ink/metadata.json ${destpath}/${addrs[i-1]}/ ;cp -f ${srcpath}${addrs[i-1]}/target/ink/*.wasm ${destpath}/${addrs[i-1]}/ ; cp -f ${srcpath}${addrs[i-1]}/target/ink/*.contract ${destpath}/${addrs[i-1]}/ ; done   


