{
  description = "Build a cargo workspace";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    { self
    , nixpkgs
    , crane
    , fenix
    , flake-utils
    , advisory-db
    , ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        inherit (pkgs) lib;
        craneLib = crane.mkLib pkgs;
        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = with pkgs; [ SDL2 ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [ pkgs.libiconv ];
        };

        craneLibLLvmTools = craneLib.overrideToolchain (
          fenix.packages.${system}.complete.withComponents [
            "cargo"
            "llvm-tools"
            "rustc"
          ]
        );

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        individualCrateArgs = commonArgs // {
          inherit cargoArtifacts;
          inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
          doCheck = false;
        };

        createFileSet =
          crate:
          lib.fileset.toSource {
            root = ./.;
            fileset = lib.fileset.unions [
              ./Cargo.toml
              ./Cargo.lock
              ./chip8
              crate
            ];
          };

        desktop = craneLib.buildPackage (
          individualCrateArgs
          // {
            pname = "desktop";
            cargoExtraArgs = "-p desktop";
            src = createFileSet ./desktop;
          }
        );
      in
      {
        checks = {
          inherit desktop;

          workspace-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          workspace-fmt = craneLib.cargoFmt { inherit src; };

          workspace-audit = craneLib.cargoAudit {
            inherit src advisory-db;
          };

          #workspace-deny = craneLib.cargoDeny {
          #  inherit src;
          #};

          workspace-nextest = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });
        };

        packages = {
          inherit desktop;
          default = desktop;
        } // lib.optionalAttrs (!pkgs.stdenv.isDarwin) {
          workspace-llvm-coverage = craneLibLLvmTools.cargoLlvmCov (commonArgs // { inherit cargoArtifacts; });
        };

        apps = {
          default = flake-utils.lib.mkApp { drv = desktop; };
          desktop = flake-utils.lib.mkApp { drv = desktop; };
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          packages = [ ];
        };
      }
    );
}
