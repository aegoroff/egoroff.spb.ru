#!/bin/bash

CONFIGURATION=$1
[[ -n "${CONFIGURATION}" ]] || CONFIGURATION="debug"
if [[ $CONFIGURATION = "release" ]]; then
  ADDITIONAL_OPTIONS="--release"
else
  ADDITIONAL_OPTIONS=""
fi

base_path=./home
[[ -d "$base_path" ]] && rm -r "$base_path"

(
  cd ./ui/
  bun run build
)

mkdir "$base_path"
LOCALS=(
  "static"
  "apache"
)
for local in "${LOCALS[@]}"; do
  cp -v -R "./$local/" "$base_path/$local/"
done

(
  cd ./egoroff/
  cargo clean
  cargo b --workspace $ADDITIONAL_OPTIONS
  ./target/$CONFIGURATION/egoroff server
)
