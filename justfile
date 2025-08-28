set export

LOG_DEFAULT      := env("LOG_DEFAULT", "debug")
LOG_BACKEND      := ",backend="      + env("LOG_BACKEND", "trace")
LOG_HAKONIWA     := ",hakoniwa="     + env("LOG_HAKONIWA", "info")
LOG_LSP          := ",backend::lsp=" + env("LOG_LSP", "off")
RUST_LOG_DEFAULT := "actix_server=off,actix_server::server=info," + LOG_DEFAULT + LOG_BACKEND + LOG_HAKONIWA + LOG_LSP
RUST_LOG         := env("RUST_LOG", RUST_LOG_DEFAULT)

[group("run")]
run-backend:
	cargo run -p backend

[group("run")]
run-frontend:
	cd frontend && pnpm dev

[group("vendor")]
@setup-vendor:
  just setup-vendor-check-binary curl
  just setup-vendor-check-binary ouch
  just setup-vendor-check-binary rsync

  #########################
  ###### Downloading ######
  #########################

  mkdir -p backend/runner/lxc_vendor

  echo -e "\x1b[1;30;43m  Downloading vendors  \x1b[0m"

  # just setup-vendor-download rust-stable-2025-03-18
  # just setup-vendor-download bash-interactive-5.2
  # just setup-vendor-download coreutils-full-9.6

  #########################
  ##### Uncompressing #####
  #########################

  echo -e "\x1b[1;30;43m  Uncompressing vendor  \x1b[0m"

  just setup-vendor-uncompress acl-2.3.2
  just setup-vendor-uncompress attr-2.5.2
  just setup-vendor-uncompress bash-interactive-5.2
  just setup-vendor-uncompress binutils-2.43.1
  just setup-vendor-uncompress binutils-2.43.1-lib
  just setup-vendor-uncompress coreutils-full-9.6
  just setup-vendor-uncompress file-5.46
  just setup-vendor-uncompress gcc-14-20241116
  just setup-vendor-uncompress gcc-14-20241116-lib
  just setup-vendor-uncompress glibc-2.40-66-lib
  just setup-vendor-uncompress glibc-2.40-66-bin
  just setup-vendor-uncompress gmp-with-cxx-6.3.0
  just setup-vendor-uncompress isl-0.20
  just setup-vendor-uncompress libmpc-1.3.1
  just setup-vendor-uncompress mpfr-4.2.1
  just setup-vendor-uncompress ncurses-6.5
  just setup-vendor-uncompress openssl-3.4.1
  just setup-vendor-uncompress patchelf-0.15.0
  just setup-vendor-uncompress readline-8.2p13
  just setup-vendor-uncompress rust-stable-2025-03-18
  just setup-vendor-uncompress xgcc-14-20241116-libgcc
  just setup-vendor-uncompress zlib-1.3.1

  ########################
  ######## ROOTFS ########
  ########################

  echo ""
  echo -e "\x1b[1;30;43m  Creating rootfs  \x1b[0m"

  if [ -d backend/runner/lxc_roofs ]; then \
    rm -rf backend/runner/lxc_roofs; \
  fi

  mkdir -p backend/runner/lxc_rootfs

  just setup-vendor-rootfs acl-2.3.2
  just setup-vendor-rootfs attr-2.5.2
  just setup-vendor-rootfs bash-interactive-5.2
  just setup-vendor-rootfs binutils-2.43.1
  just setup-vendor-rootfs binutils-2.43.1-lib
  just setup-vendor-rootfs coreutils-full-9.6
  just setup-vendor-rootfs file-5.46
  just setup-vendor-rootfs gcc-14-20241116
  just setup-vendor-rootfs gcc-14-20241116-lib
  just setup-vendor-rootfs glibc-2.40-66-lib
  just setup-vendor-rootfs glibc-2.40-66-bin
  just setup-vendor-rootfs gmp-with-cxx-6.3.0
  just setup-vendor-rootfs isl-0.20
  just setup-vendor-rootfs libmpc-1.3.1
  just setup-vendor-rootfs mpfr-4.2.1
  just setup-vendor-rootfs ncurses-6.5
  just setup-vendor-rootfs openssl-3.4.1
  just setup-vendor-rootfs patchelf-0.15.0
  just setup-vendor-rootfs readline-8.2p13
  just setup-vendor-rootfs rust-stable-2025-03-18
  just setup-vendor-rootfs xgcc-14-20241116-libgcc
  just setup-vendor-rootfs zlib-1.3.1

  echo -e "\x1b[1;30;43m  FINISH  \x1b[0m"

[group("vendor")]
@setup-vendor-if-not:
  if [ -d backend/runner/lxc_rootfs ]; then \
    echo "Already vendored"; \
  else \
    just setup-vendor; \
  fi

[private]
@setup-vendor-check-binary NAME:
  if ! command -v {{NAME}} 2>&1 >/dev/null; then \
      echo "Needs \`{{NAME}}\` command for work";\
      exit 1; \
  fi

[private]
@setup-vendor-download NAME:
  if [ -f backend/runner/lxc_vendor/{{NAME}}.tar.xz ]; then \
    echo -e "\x1b[1;32m  \`{{NAME}}.tar.xz\` already downloaded\x1b[0m"; \
  else \
    echo -e "\x1b[1;33m  Downloading \`{{NAME}}.tar.xz\`\x1b[0m"; \
    curl -L https://vendor.rsground.rustlang-es.org/{{NAME}}.tar.xz -o backend/runner/lxc_vendor/{{NAME}}.tar.xz; \
  fi

[private]
@setup-vendor-uncompress NAME:
  if [ -d backend/runner/lxc_vendor/{{NAME}} ]; then \
    rm -rf backend/runner/lxc_vendor/{{NAME}}; \
  fi

  echo -e "\x1b[1;33m  Uncompressing \`{{NAME}}\`\x1b[0m"
  cd backend/runner/lxc_vendor; \
  ouch d --no -q {{NAME}}.tar.xz

[private]
@setup-vendor-rootfs NAME:
  @rsync -avq backend/runner/lxc_vendor/{{NAME}}/ backend/runner/lxc_rootfs/
