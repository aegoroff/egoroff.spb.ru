#!/bin/bash

tag="egoroff/egoroffspbru"
DOCKER_BUILDKIT=1 docker build . -t $tag --progress plain
docker push $tag
