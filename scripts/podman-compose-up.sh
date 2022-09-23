#!/bin/bash

cd ../docker
podman-compose -f docker-compose.yml up -d
cd -