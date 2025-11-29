{ pkgs ? import <nixpkgs> {} }:

let
  rustup = pkgs.rustup;
in
pkgs.mkShell {
  buildInputs = [ rustup pkgs.cacert pkgs.openssl ];

  # Ensure rustup uses the toolchain file when the shell starts
  shellHook = ''
    if [ -f "${toString ./rust-toolchain.toml}" ]; then
      export RUSTUP_TOOLCHAIN=$(cat rust-toolchain.toml | grep "channel" | head -n1 | awk -F '"' '{print $2}')
      echo "Using Rust toolchain: $RUSTUP_TOOLCHAIN"
      rustup default "$RUSTUP_TOOLCHAIN" || true
    fi
  '';

  # Make cargo available
  RUST_BACKTRACE = "1";
}
