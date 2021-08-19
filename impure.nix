# Slimmed down shell.nix for users who have Rust pre-installed
with import <nixpkgs> { };
mkShell { buildInputs = [ pkgconfig openssl ]; }
