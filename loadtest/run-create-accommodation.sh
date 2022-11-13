#!/bin/bash

export ACCESS_TOKEN=$(curl -d 'client_id=web-app' -d 'username=u1' -d 'password=u1' -d 'grant_type=password' 'http://localhost:8080/realms/app/protocol/openid-connect/token' | jq -r '.access_token')
./gradlew --no-daemon gatlingRun-simulations.CreateAccommodationSimulation
unset ACCESS_TOKEN