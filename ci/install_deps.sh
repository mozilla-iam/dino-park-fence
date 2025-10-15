export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo
export PATH=/usr/local/cargo/bin:$PATH
export RUST_VERSION=1.69.0
export DESIRED_VERSION="v3.5.4"

set -eux
case "$(arch)" in
    x86_64) rustArch='x86_64-unknown-linux-gnu'; rustupSha256='49c96f3f74be82f4752b8bffcf81961dea5e6e94ce1ccba94435f12e871c3bdb' ;;
    aarch64) rustArch='aarch64-unknown-linux-gnu'; rustupSha256='d93ef6f91dab8299f46eef26a56c2d97c66271cea60bf004f2f088a86a697078' ;;
    *) echo >&2 "unsupported architecture: $(arch)"; exit 1 ;;
esac;
url="https://static.rust-lang.org/rustup/archive/1.22.1/${rustArch}/rustup-init";
wget --no-verbose "$url"
echo "${rustupSha256} *rustup-init" | sha256sum -c -
chmod +x rustup-init
./rustup-init -y --no-modify-path --default-toolchain "$RUST_VERSION"
rm rustup-init
chmod -R a+w $RUSTUP_HOME $CARGO_HOME
rustup --version
cargo --version
rustc --version

rustup target add "$rustArch"
HELM_INSTALL_DIR=/bin
curl https://raw.githubusercontent.com/helm/helm/master/scripts/get-helm-3 | bash
