#!/bin/bash

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

cd $SCRIPT_DIR/graphql-federation/

rover supergraph compose --config ./supergraph-config.yml >| supergraph.graphql

cd -