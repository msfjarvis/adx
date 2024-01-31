{
  description = "adx";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  inputs.systems.url = "github:msfjarvis/flake-systems";

  inputs.advisory-db.url = "github:rustsec/advisory-db";
  inputs.advisory-db.flake = false;

  inputs.crane.url = "github:ipetkov/crane";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";

  inputs.devshell.url = "github:numtide/devshell";
  inputs.devshell.inputs.nixpkgs.follows = "nixpkgs";
  inputs.devshell.inputs.flake-utils.follows = "flake-utils";

  inputs.fenix.url = "github:nix-community/fenix";
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";

  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.flake-utils.inputs.systems.follows = "systems";

  inputs.flake-compat.url = "github:nix-community/flake-compat";
  inputs.flake-compat.flake = false;

  outputs = {
    self,
    nixpkgs,
    advisory-db,
    crane,
    devshell,
    fenix,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [devshell.overlays.default];
      };

      rustStable = (import fenix {inherit pkgs;}).fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-SXRtAuO4IqNOQq+nLbrsDFbVk+3aVA8NNpSZsKlVH/8=";
      };

      craneLib = (crane.mkLib pkgs).overrideToolchain rustStable;
      xmlFilter = path: _type: builtins.match ".*xml$" path != null;
      xmlOrCargo = path: type:
        (xmlFilter path type) || (craneLib.filterCargoSources path type);

      workspaceName = craneLib.crateNameFromCargoToml {cargoToml = ./adx/Cargo.toml;};
      commonArgs = {
        inherit (workspaceName) pname version;
        src = pkgs.lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = xmlOrCargo;
        };
        buildInputs = [];
        nativeBuildInputs = [];
        cargoClippyExtraArgs = "--all-targets -- --deny warnings";
        cargoToml = ./adx/Cargo.toml;
      };

      cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {doCheck = false;});
      adx = craneLib.buildPackage (commonArgs // {doCheck = false;});
      adx-clippy = craneLib.cargoClippy (commonArgs
        // {
          inherit cargoArtifacts;
        });
      adx-fmt = craneLib.cargoFmt (commonArgs // {});
      adx-audit = craneLib.cargoAudit (commonArgs // {inherit advisory-db;});
      adx-nextest = craneLib.cargoNextest (commonArgs
        // {
          inherit cargoArtifacts;
          partitions = 1;
          partitionType = "count";
        });
    in {
      checks = {
        inherit adx adx-audit adx-clippy adx-fmt adx-nextest;
      };

      packages.default = adx;

      apps.default = flake-utils.lib.mkApp {drv = adx;};

      devShells.default = pkgs.devshell.mkShell {
        bash = {interactive = "";};

        env = [
          {
            name = "DEVSHELL_NO_MOTD";
            value = 1;
          }
        ];

        packages = with pkgs; [
          cargo-nextest
          cargo-release
          rustStable
          stdenv.cc
        ];
      };
    });
}
