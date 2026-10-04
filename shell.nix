{ pkgs ? import <nixpkgs> { } }:

assert pkgs.lib.assertMsg (pkgs.lib.versionAtLeast pkgs.rustc.version "1.96")
  "FastSecDec's Symbolica dependency requires Rust 1.96 or newer.";

pkgs.mkShell {
  packages = with pkgs; [ cargo rustc rustfmt clippy pkg-config gnumake m4 ];
}
