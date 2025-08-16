#!/bin/bash

tag="ghcr.io/aegoroff/egoroff.spb.ru:master"
DOCKER_BUILDKIT=1 docker build . -t $tag --progress plain
docker push $tag
DOCKER_BUILDKIT=1 docker build . -f DockerfileArm64 -t "${tag}-arm64" --progress plain --platform=linux/arm64
docker push "${tag}-arm64"