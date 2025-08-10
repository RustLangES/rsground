# Things to know
The prefix `Rg` (Ej. `RgWebsocket`) means for `RsGround`, a way to difference business structs

# Execute with justfile 

```sh
$ just run-backend
```

| env_var | description |
|---|---|
| `LOG_DEFAULT` | set any logs level |
| `LOG_BACKEND` | set RsGround backend logs level |
| `LOG_LSP` | set Rust-analyzer logs level |
| `LOG_HAKONIWA` | set LXC logs level |

# Manual execution
To see logs you need to setup what you want in `RUST_LOG` env var with the following format:

`RUST_LOG=[target=]<level>`

| target | description |
|---|---|
| _No target_ | Any logs |
| `actix_server` | Actix lib |
| `backend` | RsGround backend |
| `backend::auth` | Authorization |
| `backend::collab` | Multiplayer syncronization |
| `backend::lsp` | Rust-analyzer |
| `backend::producer` | Compilation |
| `backend::project` | Project managment |
| `backend::ws` | Websocket |
| `hakoniwa` | LXC logs |

Level is the maximum log::Level to be shown and includes:
- off
- error
- warn
- info
- debug
- trace

Recommended logs:
```sh
RUST_LOG=debug,actix_server=off,actix_server::server=info,backend=trace,backend::lsp=debug,hakoniwa=info
```

# Tests
> [!NOTE]
> Before any test related to the backend, start the server.

Execute tests:
```sh
$ cargo test --no-fail-fast --test integration_test

$ cargo test --no-fail-fast --bin backend --verbose

$ cargo test --no-fail-fast -p rsground-runner --verbose
```

# Websocket connection workflow
```mermaid
---
config:
    theme: dark
    look: classic
    layout: elk
---
flowchart TD
    subgraph JoinProject
    JoinProject1["User want to join"] --> JoinProject2{"Has access?"}
    JoinProject2 -->|YES| JoinProject3("Send actual access level")
    JoinProject2 -->|NO| JoinProject4{Is public room?}
    JoinProject4 -->|NO| JoinProject5(Pending request)
    JoinProject4 -->|YES| JoinProject6{"Has correct password?"}
    JoinProject6 -->|NO| JoinProject7(Invalid password)
    JoinProject6 -->|YES| JoinProject8(Allow read access)
    JoinProject8 --> JoinProject9(Broadcast updated access)

    JoinProject3 --> JoinProject10("Sync others access level")
    JoinProject9 --> JoinProject10
    end

    subgraph PermitAccess-V1
    PermitAccess1["User change access level of other"] --> PermitAccess2{Is the owner?}
    PermitAccess2 -->|NO| PermitAccess3("Err(NotOwner)")
    PermitAccess2 -->|YES| PermitAccess4("Update user access")
    PermitAccess4 --> PermitAccess5("Broadcast updated acess")
    end
```
