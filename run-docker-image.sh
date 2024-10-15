#!/usr/bin/env bash

docker run -d -p 8090:8090 --env-file=.env crash-server:latest