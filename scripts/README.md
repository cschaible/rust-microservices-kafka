# Rover cli

To update the supergraph schema the rover cli from apollo is needed to merge the
schemas of the different services.

## Install

### Install on mac
```shell
brew install rover
```

### Install on other platforms
https://www.apollographql.com/docs/rover/

## Run 
```shell
./apollo_build_supergraph.sh
```

# Router

The apollo router needs to be downloaded or installed on the local machine.

## Install

### Install on apple silicon (m1, m2)

There are no pre-build binaries for arm platform yet. Therefore, the router
needs to be compiled locally.

Execute the following scripts to download and compile:
```shell
./apollo_router_download.sh
./apollo_router_build.sh
```

### Install on other  platforms
https://www.apollographql.com/docs/router/quickstart/

## Run
```shell
./apollo_router_run.sh
```