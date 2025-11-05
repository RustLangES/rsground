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
    architectures = import ./architectures.nix;

    rustToolchainDef = {
      channel = "1.90.0";
      sha256 = "sha256-SJwZ8g0zF2WrKDVmHrVG3pD2RGoQeo24MEXnNx5FyuI=";
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

    mkPackage = { os, ... } @ variant: let
      crossPkgs = mkCrossPkgs variant;
    in (pkgs.makeRustPlatform {
      inherit (toolchain) cargo rustc;
    }).buildRustPackage (finalAttrs: {
      doCheck = false;
      pname = "backend";
      version = cargoManifest.package.version;

      cargoBuildFlags = ["-p" "backend"];

      src = ./.;
      cargoLock.lockFile = ./Cargo.lock;

      env = {
        OPENSSL_NO_VENDOR = 1;
        HOST_CC = lib.optionalString (os != "windows") "${pkgs.stdenv.cc.nativePrefix}cc";
        TARGET_CC = lib.optionalString (os != "windows") "${crossPkgs.stdenv.cc.targetPrefix}cc";
      };

      nativeBuildInputs = [pkgs.pkg-config];

      buildInputs = with pkgs; [
        openssl
      ] ++ lib.optionals stdenv.buildPlatform.isDarwin [
        libiconv
        cctools.libtool
      ];
    });

    mkCrossPkgs = { arch, os, ... }: let
      cross = arch + "-" + os;
      crossSystem = lib.systems.elaborate cross;
    in import nixpkgs {
      crossSystem = if cross != "x86_64-linux" then crossSystem else null;
      localSystem = system;
    };

    containerPkg = { arch, ... } @ variant: let
      appPkg = mkPackage variant;
      dockerPlatform =
        if arch == "x86_64" then "amd64"
        else if arch == "aarch64" then "arm64"
        else if arch == "armv7l" then "arm"
        else if arch == "armv6l" then "arm"
        else if arch == "i686" then "386"
        else throw "Unsupported arch: ${arch}";
    in pkgs.dockerTools.buildLayeredImage {
      created = "now";
      name = "rsground";
      tag = cargoManifest.package.version;
      architecture = dockerPlatform;

      contents = [ appPkg ];
      config.Cmd = ["/bin/backend"];
    };

    generatedMatrixJson = builtins.toJSON (lib.flatten (map ({ arch, os, ... }: {
      inherit os arch;
      package = "${os}-${arch}";
      version = cargoManifest.package.version;
    }) architectures));
  in {
    apps.${system}.matrix = {
      type = "app";
      program = toString (pkgs.writeScript "generate-matrix" ''
        #!/bin/sh
        echo '${generatedMatrixJson}'
      '');
      meta = {
          description = "support matrix for CI";
      };
    };

    packages.${system} =
      let
        perArch =
          lib.listToAttrs (map (variant: {
            name = "image-${variant.os}-${variant.arch}";
            value = containerPkg variant;
          }) architectures);

        perArchPkgs =
          lib.listToAttrs (map (variant: {
            name = "backend-${variant.os}-${variant.arch}";
            value = mkPackage variant;
          }) architectures);
      in
        perArch // perArchPkgs // {
          docker-manifest = pkgs.writeShellScriptBin "docker-manifest" ''
            set -euo pipefail
            VERSION=${cargoManifest.package.version}
            IMAGE="ghcr.io/$REPOSITORY:$VERSION"

            docker manifest create "$IMAGE" ${lib.concatMapStringsSep " " (variant:
              "--amend ghcr.io/$REPOSITORY:$VERSION-${variant.os}-${variant.arch}"
            ) architectures}
            docker manifest push "$IMAGE"
          '';
        };

    devShells.${system}.default = pkgs.mkShell {
      packages =
        commonBuildInputs
        ++ backendBuildInputs
        ++ frontendBuildInputs;

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
