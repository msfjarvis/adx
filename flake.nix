{
  description = "adx";

  inputs.nixpkgs.url = "github:msfjarvis/nixpkgs/nixpkgs-unstable";

  inputs.systems.url = "github:msfjarvis/flake-systems";

  inputs.advisory-db.url = "github:rustsec/advisory-db";
  inputs.advisory-db.flake = false;

  inputs.crane.url = "github:ipetkov/crane";

  inputs.devshell.url = "github:numtide/devshell";
  inputs.devshell.inputs.nixpkgs.follows = "nixpkgs";

  inputs.fenix.url = "github:nix-community/fenix";
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.flake-utils.inputs.systems.follows = "systems";

  inputs.flake-compat.url = "git+https://git.lix.systems/lix-project/flake-compat";
  inputs.flake-compat.flake = false;

  outputs =
    {
      nixpkgs,
      advisory-db,
      crane,
      devshell,
      fenix,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlays.default ];
        };

        rustStable = (import fenix { inherit pkgs; }).fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-sqSWJDUxc+zaz1nBWMAJKTAGBuGWP25GCftIOlCEAtA=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustStable;

        workspaceName = craneLib.crateNameFromCargoToml { cargoToml = ./adx/Cargo.toml; };
        commonArgs = {
          inherit (workspaceName) pname version;
          src =
            with pkgs.lib.fileset;
            toSource {
              root = ./.;
              fileset = unions [
                (fileFilter (file: file.hasExt "xml") ./.)
                (fileFilter (file: file.hasExt "snap") ./.)
                (craneLib.fileset.commonCargoSources ./.)
              ];
            };
          buildInputs = [ ];
          nativeBuildInputs = pkgs.lib.optionals pkgs.stdenv.buildPlatform.isDarwin [ pkgs.darwin.libiconv ];
          cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          cargoToml = ./adx/Cargo.toml;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        adx-build = craneLib.cargoBuild (
          commonArgs
          // {
            inherit cargoArtifacts;
            CARGO_BUILD_RUSTFLAGS = "--cfg feature=\"nix-check\"";
          }
        );
        adx = craneLib.buildPackage (
          commonArgs
          // {
            cargoArtifacts = adx-build;
            doCheck = false;
          }
        );
        adx-clippy = craneLib.cargoClippy (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
        adx-fmt = craneLib.cargoFmt (commonArgs // { });
        adx-audit = craneLib.cargoAudit (commonArgs // { inherit advisory-db; });
        adx-nextest = craneLib.cargoNextest (
          commonArgs
          // {
            cargoArtifacts = adx-build;
            partitions = 1;
            partitionType = "count";
            CARGO_BUILD_RUSTFLAGS = "--cfg feature=\"nix-check\"";
          }
        );
      in
      {
        checks = {
          inherit
            adx
            adx-audit
            adx-clippy
            adx-fmt
            adx-nextest
            ;
        };

        packages.default = adx;

        apps.default = flake-utils.lib.mkApp { drv = adx; };

        devShells.default = pkgs.devshell.mkShell {
          imports = [
            "${devshell}/extra/language/c.nix"
            "${devshell}/extra/language/rust.nix"
          ];
          bash = {
            interactive = "";
          };

          env = [
            {
              name = "DEVSHELL_NO_MOTD";
              value = 1;
            }
          ];

          packages = with pkgs; [
            cargo-dist
            cargo-insta
            cargo-nextest
            cargo-release
            fenix.packages.${system}.rust-analyzer
            oranda
            rustStable
            stdenv.cc
          ];

          language.c.libraries = commonArgs.nativeBuildInputs;
          language.rust.enableDefaultToolchain = false;
        };
      }
    );
}
