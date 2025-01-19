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

  inputs.flake-compat.url = "github:nix-community/flake-compat";
  inputs.flake-compat.flake = false;

  outputs =
    {
      self,
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
          sha256 = "sha256-lMLAupxng4Fd9F1oDw8gx+qA0RuF7ou7xhNU8wgs0PU=";
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain rustStable;
        xmlFilter = path: builtins.match ".*xml$" path != null;
        instaFilter = path: builtins.match ".*snap$" path != null;
        filter =
          path: type: (xmlFilter path) || (instaFilter path) || (craneLib.filterCargoSources path type);

        workspaceName = craneLib.crateNameFromCargoToml { cargoToml = ./adx/Cargo.toml; };
        commonArgs = {
          inherit (workspaceName) pname version;
          src = pkgs.lib.cleanSourceWith {
            src = craneLib.path ./.;
            inherit filter;
          };
          buildInputs = [ ];
          nativeBuildInputs = [ ];
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
        };
      }
    );
}
