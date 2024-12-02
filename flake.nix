{
  description = "Conc is a simple desktop process manager.";

  # GUI is currently not provided, since it requires some other dependencies & dymanic linking on Nix
  # See https://github.com/rust-windowing/winit/issues/3603

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        version = "0.0.1";
      in
      rec {
        packages.daemon = pkgs.stdenv.mkDerivation {
          pname = "concd";
          version = version;
          src = ./apps/daemon;

          outputs = [ "out" ];

          hardeningDisable = [ "all" ];

          buildInputs = with pkgs; [
            gnumake
            gcc
          ];

          # Remove bash shebang, since it seems to break things
          preInstall = ''
            sed -i -e "s@#! /usr/bin/env bash@@" -e "s@sudo @@g" install.sh
            export PREFIX="$out/bin" SYSTEMD_PREFIX="$out/systemd" HOME="$out/home"
            mkdir -p $PREFIX
          '';

          meta = {
            homepage = "https://github.com/prixladi/conc";
            description = "Daemon for conc - process manager";
            license = pkgs.lib.licenses.mit;
            maintainers = [ ];
            platforms = pkgs.lib.platforms.all;
            mainProgram = "concd";
          };
        };

        packages.cli = pkgs.rustPlatform.buildRustPackage {
          pname = "concc";
          version = version;
          src = ./.;
          buildAndTestSubdir = "apps/cli";

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          hardeningDisable = [ "all" ];

          meta = {
            homepage = "https://github.com/prixladi/conc";
            description = "CLI for conc - process manager";
            license = pkgs.lib.licenses.mit;
            maintainers = [ ];
            platforms = pkgs.lib.platforms.all;
            mainProgram = "concc";
          };
        };

        services.daemon = {
          unitConfig = {
            Description = "Conc service daemon";
          };

          serviceConfig = {
            WorkingDirectory = "%h/.conc/run";
            ExecStart = ''
              ${packages.daemon.out}/bin/concd
            '';
          };
          wantedBy = [ "multi-user.target" ];
        };
      }
    );
}
