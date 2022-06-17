with import <nixpkgs> { overlays = [ (import <rust-overlay>) ]; };
mkShell {
  buildInputs = [
    (rust-bin.stable.latest.default.override {
      extensions = [
        "clippy"
        "llvm-tools-preview"
        "rust-src"
        "rustc-dev"
        "rustfmt-preview"
      ];
      targets = lib.optionals stdenv.isDarwin [ "aarch64-apple-darwin" ]
        ++ lib.optionals stdenv.isLinux [ "x86_64-unknown-linux-gnu" ];
    })
  ] ++ lib.optionals stdenv.isDarwin [
    pkgs.darwin.apple_sdk.frameworks.Security
  ];
}
