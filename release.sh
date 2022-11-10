set -ex
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-pc-windows-gnu

(cd target/x86_64-unknown-linux-gnu/release/ && zip ../../../linux.zip model-script)
(cd target/x86_64-pc-windows-gnu/release/ && zip ../../../windows.zip model-script.exe)
