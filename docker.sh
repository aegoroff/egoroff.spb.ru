#!/bin/bash

tag="egoroff/egoroffspbru"
docker build . -t $tag
docker push $tag
