#!/usr/bin/env bash

mkdir -p .cargo

case $1 in
"dev")
cp envs/dev .cargo/config
;;

"ci")
cp envs/ci .cargo/config
;;

*)
echo "use-env.sh [dev|ci]"
exit 1

esac