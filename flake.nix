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
  in {
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
