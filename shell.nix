{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  packages = with pkgs; [
    rustc
    cargo
    gcc
    rustfmt
    clippy
    rust-analyzer
  ];

  RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

  shellHook = ''
    export PATH="$HOME/.cargo/bin:$PATH"
  '';
}
