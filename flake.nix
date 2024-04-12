# From https://github.com/litchipi/nix-build-templates/blob/6e4961dc56a9bbfa3acf316d81861f5bd1ea37ca/rust/maturin.nix
# See also https://discourse.nixos.org/t/pyo3-maturin-python-native-dependency-management-vs-nixpkgs/21739/2
{
  # Build Pyo3 package
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-23.05";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [ inputs.rust-overlay.overlays.default ];
        };
        lib = pkgs.lib;

        # Get a custom rust toolchain
        customRustToolchain = pkgs.rust-bin.stable."1.70.0".default;
        craneLib =
          (inputs.crane.mkLib pkgs).overrideToolchain customRustToolchain;

        projectName =
          (craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; }).pname;
        projectVersion = (craneLib.crateNameFromCargoToml {
          cargoToml = ./Cargo.toml;
        }).version;

        allDeps = with pkgs; [ openssl ];

        # Clean source, keeping necessary extra files
        texFilter = path: _type: builtins.match ".*tex$" path != null;
        clsFilter = path: _type: builtins.match ".*cls$" path != null;
        finalSourceFilter = path: type:
          (clsFilter path type) || (texFilter path type)
          || (craneLib.filterCargoSources path type);

        cleanedSource = lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = finalSourceFilter;
        };

        crateCfg = {
          src = cleanedSource;
          nativeBuildInputs = allDeps;
        };

        # Build the library, then re-use the target dir to generate the wheel file with maturin
        crate = (craneLib.buildPackage (crateCfg // {
          pname = projectName;
          version = projectVersion;
          # cargoArtifacts = crateArtifacts;
        }));

      in rec {
        packages = rec {
          inherit crate;
          default = crate; # The wheel itself
        };

        devShell = devShells.default;
        devShells = rec {
          rust = pkgs.mkShell {
            name = "rust-env";
            src = ./.;
            nativeBuildInputs =
              (with pkgs; [ pkg-config rust-analyzer maturin ]) ++ allDeps;
          };
          default = rust;
        };

        apps = rec {
          pdoc = {
            type = "app";
            program = "${crate}/bin/pdoc";
          };
        };
      });
}
