{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        rustPlatform = pkgs.rustPlatform;
      in {
        defaultPackage = rustPlatform.buildRustPackage {
          pname = "invgen";
          version = "0.1.0";

          nativeBuildInputs = with pkgs; [ lld pkgconfig openssl udev ];

          cargoLock = { lockFile = ./Cargo.lock; };

          src = ./.;
        };

        devShell = pkgs.mkShell {
          name = "invgen-shell";
          src = ./.;

          buildInputs = with pkgs; [
            openssl
            icu
            graphite2
            freetype
            fontconfig
          ];

          # build-time deps
          nativeBuildInputs = with pkgs; [ rustc cargo lld pkgconfig udev ];
        };
      });
}
