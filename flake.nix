{
  description = "Crash Server Dev Shell";

  # Flake inputs
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.2405.*.tar.gz";
    rust-overlay.url = "github:oxalica/rust-overlay"; # A helper for Rust + Nix
  };

  # Flake outputs
  outputs = { self, nixpkgs, rust-overlay }:
    let
      # Overlays enable you to customize the Nixpkgs attribute set
      overlays = [
        # Makes a `rust-bin` attribute available in Nixpkgs
        (import rust-overlay)
        # Provides a `rustToolchain` attribute for Nixpkgs that we can use to
        # create a Rust environment
        (self: super: {
          rustToolchain = super.rust-bin.stable.latest.default;
        })
      ];

      # Systems supported
      allSystems = [
        "x86_64-linux" # 64-bit Intel/AMD Linux
        "aarch64-linux" # 64-bit ARM Linux
        "x86_64-darwin" # 64-bit Intel macOS
        "aarch64-darwin" # 64-bit ARM macOS
      ];

      # Helper to provide system-specific attributes
      forAllSystems = f: nixpkgs.lib.genAttrs allSystems (system: f {
        pkgs = import nixpkgs { inherit overlays system; };
        system = system;
      });
    in
    {
      # Build
      packages = forAllSystems ({ pkgs, system }: {
        crash-server =
          let
            rustPlatform = pkgs.makeRustPlatform {
              cargo = pkgs.rustToolchain;
              rustc = pkgs.rustToolchain;
            };
          in
          rustPlatform.buildRustPackage {
            name = "crash-server";
            src = ./.;
            cargoLock = {
              lockFile = ./Cargo.lock;
            };
            nativeBuildInputs = with pkgs; [
              flatbuffers
            ];
            preBuild = ''
            ./generate-schema.sh
            '';
          };

        dockerImage =
          let
            crash_server_pkg = self.packages.${system}.crash-server;
          in
          pkgs.dockerTools.buildImage {
            name = "crash-server";
            tag = "latest";
            copyToRoot = [ crash_server_pkg ];
            config = {
              Cmd = ["${crash_server_pkg}/bin/crash-server"];
              Env = [
                "ENV=PRODUCTION"
                "RUST_LOG=DEBUG"
              ];
              ExposedPorts = {
                "8090/tcp" = {};
              };
            };
          };
      });

      # Development environment output
      devShells = forAllSystems ({ pkgs, system }: {
        default = pkgs.mkShell {
          # The Nix packages provided in the environment
          packages = (with pkgs; [
            # The package provided by our custom overlay. Includes cargo, Clippy, cargo-fmt,
            # rustdoc, rustfmt, and other tools.
            rustToolchain
            flatbuffers
          ]) ++ pkgs.lib.optionals pkgs.stdenv.isDarwin (with pkgs; [ libiconv ]);

          shellHook = ''
          echo "Hello shell!"
          export RUST_LOG=debug
          ./generate-schema.sh
          '';
        };
      });

      # default build
      defaultPackage = forAllSystems ({ pkgs, system }: self.packages.${system}.crash-server);
    };
}
