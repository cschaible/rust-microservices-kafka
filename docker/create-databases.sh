#!/bin/bash

set -e

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE USER demouser WITH PASSWORD 'secret';
    CREATE DATABASE demo;
    GRANT ALL PRIVILEGES ON DATABASE demo TO demouser;
    \c demo
    GRANT ALL ON SCHEMA public TO demouser;
    CREATE USER keycloak WITH PASSWORD 'secret';
    CREATE DATABASE keycloak;
    GRANT ALL PRIVILEGES ON DATABASE keycloak TO keycloak;
    \c keycloak
    GRANT ALL ON SCHEMA public TO keycloak;
EOSQL