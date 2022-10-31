#!/bin/bash
docker-compose -f ../docker/docker-compose.yml down

docker volume prune -f
docker system prune -f