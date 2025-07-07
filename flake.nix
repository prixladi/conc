{
  description = "Conc is a simple desktop process manager.";

  # GUI is currently not provided, since it requires some other dependencies & dymanic linking on Nix
  # See https://github.com/rust-windowing/winit/issues/3603

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        version = "0.0.1";
      in
      rec {
        devShells.default =
          with pkgs;
          mkShell rec {
            hardeningDisable = [ "all" ];
            buildInputs =
              packages.daemon.buildInputs
              ++ (with pkgs; [
                rust-bin.stable."1.85.0".default

                libxkbcommon
                libGL

                # WINIT_UNIX_BACKEND=x11
                xorg.libXcursor
                xorg.libXrandr
                xorg.libXi
                xorg.libX11
              ]);

            LD_LIBRARY_PATH = "${lib.makeLibraryPath buildInputs}";
            WINIT_UNIX_BACKEND = "x11";
          };

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
          wantedBy = [ "default.target" ];
        };
      }
    );
}
