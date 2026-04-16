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
  cd ./ui/ || exit
  bun run build || exit
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
  cd ./egoroff/ || exit
  cargo clean
  cargo b --workspace $ADDITIONAL_OPTIONS
  RUST_LOG="server=debug,axum=info,hyper=info,tower=info,axum_login=info" ./target/"$CONFIGURATION"/egoroff server
)
