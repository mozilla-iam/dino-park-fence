export RUSTUP_HOME=/usr/local/rustup
export CARGO_HOME=/usr/local/cargo
export PATH=/usr/local/cargo/bin:$PATH
export RUST_VERSION=%%RUST-VERSION%%

set -eux;
url="https://static.rust-lang.org/rustup/archive/%%RUSTUP-VERSION%%/${rustArch}/rustup-init"
wget "$url"
echo "${rustupSha256} *rustup-init" | sha256sum -c -
chmod +x rustup-init
./rustup-init -y --no-modify-path --default-toolchain $RUST_VERSION
rm rustup-init
chmod -R a+w $RUSTUP_HOME $CARGO_HOME
rustup --version
cargo --version
rustc --version

rustup target add x86_64-unknown-linux-musl
apt-get update && apt-get install -y musl-tools
rm -rf /var/lib/apt/lists/*
ENV K8SVERSION=v1.11.5
curl -LO https://storage.googleapis.com/kubernetes-release/release/$K8SVERSION/bin/linux/amd64/kubectl
chmod +x ./kubectl
mv ./kubectl /bin/kubectl
HELM_INSTALL_DIR=/bin
curl https://raw.githubusercontent.com/helm/helm/master/scripts/get | bash
curl -L -o myke https://github.com/fiji-flo/myke/releases/download/0.9.11/myke-0.9.11-x86_64-unknown-linux-musl
chmod +x ./myke
mv ./myke /bin/myke