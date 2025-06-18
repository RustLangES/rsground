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

    toolchain = fenix.toolchainOf {
      channel = "1.85.0";
      sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
    };
    wasmToolchain = fenix.targets.wasm32-unknown-unknown.toolchainOf {
      channel = "1.85.0";
      sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
    };

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
        cd frontend && pnpm install --frozen-lockfile || pnpm install
        just setup-vendor-if-not
      '';
    };
  };
}
