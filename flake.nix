{
  description = "stromer2mqtt";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      self,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      {

        packages = {
          # regular, host-native build (dynamic)
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "stromer2mqtt";
            version = cargoToml.package.version;

            src = ./.;
            cargoLock.lockFile = ./Cargo.lock;
          };
        }
        // (
          # musl-static package, only on Linux
          if pkgs.stdenv.isLinux then
            {
              static = pkgs.pkgsStatic.rustPlatform.buildRustPackage {
                pname = "stromer2mqtt";
                version = cargoToml.package.version;

                src = ./.;

                cargoLock.lockFile = ./Cargo.lock;
              };
            }
          else
            { }
        )
        // {
          dockerImage =
            let
              # Select the appropriate package (static for Linux, default otherwise)
              stromer2mqttApp =
                if pkgs.stdenv.isLinux then self.packages.${system}.static else self.packages.${system}.default;

              # Create an FHS-like environment for the application.
              # This ensures the binary is at a predictable path like /app/bin/stromer2mqtt
              # along with its runtime dependencies.
              appFhs = pkgs.buildEnv {
                name = "stromer2mqtt-fhs";
                paths = [
                  stromer2mqttApp
                  pkgs.cacert
                ];
                postBuild = ''
                  mkdir -p $out/app/bin
                  ln -s ${stromer2mqttApp}/bin/stromer2mqtt $out/app/bin/stromer2mqtt
                '';
              };

              app = pkgs.buildEnv {
                name = "stromer2mqtt";
                paths = [
                  appFhs
                ];
              };
            in
            pkgs.dockerTools.buildImage {
              name = "stromer2mqtt";
              tag = "latest";
              copyToRoot = app;

              config = {
                Cmd = [ "/app/bin/stromer2mqtt" ];
              };
            };
        };
      }
    );
}
