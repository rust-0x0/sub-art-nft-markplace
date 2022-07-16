#!/usr/bin/env bash

VERSION=0.1
WORK_DIR=$(cd $(dirname $0); pwd)
ERC1155_MODULE=erc1155
HEXSPACE_MODULE=hex_space

function build_module() {
    m_name=$1
    m_dir=${WORK_DIR}/${m_name}
    echo "build module ${m_dir}"
    cd ${m_dir}
    cargo +nightly contract build
    if [ $? -ne 0 ];then
      echo "build module failed"
      exit 1
    fi
    echo "copy to ../release"
    cp ${m_dir}/target/ink/${m_name}.wasm ../release/${m_name}_v$VERSION.wasm
    cp ${m_dir}/target/ink/${m_name}.contract ../release/${m_name}_v$VERSION.contract
    cp ${m_dir}/target/ink/metadata.json ../release/${m_name}_v$VERSION.json
    cd -
}

echo "clean release"
rm -rf ${WORK_DIR}/release
mkdir -p ${WORK_DIR}/release

build_module ${ERC1155_MODULE}
build_module ${HEXSPACE_MODULE}

