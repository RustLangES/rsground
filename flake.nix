{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";

    fenix-pkg.url = "github:nix-community/fenix";
    fenix-pkg.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = {
    nixpkgs,
    fenix-pkg,
    ...
  }: let
    system = "x86_64-linux";
    cargoManifest = builtins.fromTOML (builtins.readFile ./backend/Cargo.toml);
    pkgs = import nixpkgs {inherit system;};
    lib = pkgs.lib;
    fenix = fenix-pkg.packages.${system};

    rustToolchainDef = {
      channel = "1.88.0";
      sha256 = "sha256-Qxt8XAuaUR2OMdKbN4u8dBJOhSHxS+uS06Wl9+flVEk=";
    };
    toolchain = fenix.toolchainOf rustToolchainDef;
    wasmToolchain = fenix.targets.wasm32-unknown-unknown.toolchainOf rustToolchainDef;

    commonBuildInputs = with pkgs; [
      just
    ];

    backendBuildInputs = with pkgs; [
      (fenix.combine [
        (toolchain.withComponents ["rustc" "cargo" "rust-src" "clippy"])
        wasmToolchain.rust-std
      ])

      openssl

      # LXC vendor
      ouch
      rsync
    ];

    frontendBuildInputs = with pkgs; [
      nodejs_22
      pnpm_10
      vtsls

      wasm-pack
    ];

    appPkg = (pkgs.makeRustPlatform {
      inherit (toolchain) cargo rustc;
    }).buildRustPackage (finalAttrs: {
      doCheck = false;
      pname = "backend";
      version = cargoManifest.package.version;

      cargoBuildFlags = ["-p" "backend"];

      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      env.OPENSSL_NO_VENDOR = 1;

      nativeBuildInputs = [pkgs.pkg-config];

      buildInputs = with pkgs; [
        openssl
      ] ++ lib.optionals stdenv.buildPlatform.isDarwin [
        libiconv
        cctools.libtool
      ];
    });

    containerPkg = pkgs.dockerTools.buildLayeredImage {
      name = "rsground";
      tag = cargoManifest.package.version;
      created = "now";
      architecture = "amd64";

      contents = [ appPkg ];
      config.Cmd = ["/bin/backend"];
    };
  in {
    packages.${system} = {
        default = appPkg;
        image = containerPkg;
    };
    devShells.${system}.default = pkgs.mkShell {
      buildInputs =
        commonBuildInputs
        ++ backendBuildInputs
        ++ frontendBuildInputs;

      nativeBuildInputs = [pkgs.pkg-config];

      LIBSECCOMP_LIB_PATH = "${lib.makeLibraryPath [pkgs.libseccomp]}";
      LD_LIBRARY_PATH = "${lib.makeLibraryPath [pkgs.libseccomp]}";

      shellHook = ''
        if [ ! -e frontend-wasm/pkg ]; then wasm-pack build frontend-wasm; fi
        pnpm install -C frontend --frozen-lockfile || pnpm install -C frontend
        just setup-vendor-if-not
      '';
    };
  };
}
