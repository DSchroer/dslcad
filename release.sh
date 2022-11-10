set -ex
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu

(cd target/x86_64-unknown-linux-gnu/release/ && zip ../../../linux.zip model-script-cli)
(cd target/x86_64-pc-windows-gnu/release/ && cp /usr/x86_64-w64-mingw32/bin/*.dll . && zip ../../../windows.zip model-script-cli.exe *.dll)
