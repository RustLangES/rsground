si quieres ver los logs puedes usar

`RUST_LOG=trace`

level is the maximum log::Level to be shown and includes:
    error
    warn
    info
    debug
    trace
    off (pseudo level to disable all logging for the target)
suelo usar:
`RUST_LOG=debug cargo run --bin backend`
Para ejecutar los tests de integracion ejecute la api primero y luego
```
cargo run --bin backend
```
para ejecutar los tests
```
cargo test --test integration_test
```

## Things to know
The prefix `Rg` (Ej. `RgWebsocket`) means for `RsGround`, a way to difference business structs

## Websocket workflow
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
