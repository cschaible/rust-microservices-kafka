#!/bin/bash

cd ../docker
docker-compose -f docker-compose.yml up -d
docker rm minit
cd -