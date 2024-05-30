#!/bin/bash

tag="ghcr.io/aegoroff/egoroff.spb.ru:master"
DOCKER_BUILDKIT=1 docker build . -t $tag --progress plain
docker push $tag
