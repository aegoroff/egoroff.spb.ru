#!/bin/bash

tag="egoroff/egoroffspbru"
DOCKER_BUILDKIT=1 docker build . -t $tag
docker push $tag
