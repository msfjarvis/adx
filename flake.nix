{
  description = "adx";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

  inputs.systems.url = "github:msfjarvis/flake-systems";

  inputs.advisory-db.url = "github:rustsec/advisory-db";
  inputs.advisory-db.flake = false;

  inputs.crane.url = "github:ipetkov/crane";
  inputs.crane.inputs.flake-compat.follows = "flake-compat";
  inputs.crane.inputs.flake-utils.follows = "flake-utils";
  inputs.crane.inputs.nixpkgs.follows = "nixpkgs";

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
    fenix,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {inherit system;};

      rustStable = (import fenix {inherit pkgs;}).fromToolchainFile {
        file = ./rust-toolchain.toml;
        sha256 = "sha256-gdYqng0y9iHYzYPAdkC/ka3DRny3La/S5G8ASj0Ayyc=";
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

      devShells.default = pkgs.mkShell {
        inputsFrom = builtins.attrValues self.checks.${system};

        nativeBuildInputs = with pkgs; [
          cargo-nextest
          cargo-release
          rustStable
        ];

        CARGO_REGISTRIES_CRATES_IO_PROTOCOL = "sparse";
      };
    });
}
