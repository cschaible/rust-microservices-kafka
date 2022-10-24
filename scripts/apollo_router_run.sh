#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd $SCRIPT_DIR/../router/target/release/

./router --config $SCRIPT_DIR/graphql-federation/config.yml --supergraph $SCRIPT_DIR/graphql-federation/supergraph.graphql --log info

cd -