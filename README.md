# flare-grpc-proto

English · [中文](README.zh-CN.md)

> ## ℹ This is communication infrastructure, not a ready-to-use IM product
>
> Up front, so you don't clone it only to find you can't log in: **the
> open-source part does not include an account system** (no sign-up/login,
> no friend relationships, no group roles/approval/muting, no moments feed).
>
> But it ships with a complete, pluggable authentication contract, and both
> paths live on the open-source side:
>
> - **`CoreJwtTokenValidator`** — validates JWTs locally. Hand-sign a token
>   and you can get it running for a demo / POC, **with no user system at all**.
> - **`HttpHookTokenValidator`** — POSTs the token to your own endpoint;
>   **this is the entry point for integrating your own user system**.
>
> Business rules work the same way: `flare-core-server/crates/flare-im-hooks`
> provides 9 extension points (PreSend / PostSend / Delivery / Recall /
> MessageRead / MessageReaction / ConversationLifecycle / ConversationMember /
> GetConversationParticipants).
>
> To go to production, you implement your own user system and wire it in via
> the contracts above — the same "bring your own identity" model as Sendbird /
> Twilio Conversations, except Flare can be self-hosted and its protocol and
> core are auditable.
>
> See [GOVERNANCE.md](.github/GOVERNANCE.md) for the boundary details.


The **gRPC service-layer Protocol Buffers** and **Rust generated code**
(`tonic` + `prost`) for Flare IM. Business service crates depend on this
library to obtain each service's client / server types and message definitions.

## Responsibility boundary

| Component | Description |
|------|------|
| **`flare-proto`** | Domain models and common types (such as `flare.common.v1`, the `Message` body, etc.) |
| **This crate** | The `.proto` files and compiled artifacts for each gRPC service; references the common packages in `flare_proto` via `extern_path` to avoid duplicate generation |

## Directory structure

```text
flare-grpc-proto/
├── proto/           # Service definitions (*.proto)
├── build.rs         # tonic-prost-build compilation entry point
├── src/lib.rs       # include_proto! and convenience re-exports
├── Cargo.toml
└── README.md
```

## Service Protos in this repository

The files currently compiled by `build.rs` (order matters for dependency resolution):

- `access_gateway.proto`
- `conversation_service.proto`
- `capability_service.proto` (package `flare.capability.v1`: `CapabilityService`, `HookExtension`, `HookService`)
- `media_service.proto`
- `message_service.proto`
- `online.proto`
- `push_service.proto`
- `router.proto`
- `storage_service.proto`
- `sync_service.proto`

The `import` statements for base types are provided by **`flare-protocol/proto`**;
at compile time the include path is "base directory first, then this directory",
to reduce ambiguity over identically named imports.

## Dependencies

- **Rust**: see `rust-version` in the root `Cargo.toml` (currently **1.94+**, Edition **2024**).
- **crates**: `flare-proto`, `prost`, `tonic`, `serde`, and others — see `Cargo.toml` for details.

> The path to `flare-proto` is authoritatively defined in **`Cargo.toml`**; it may use a relative path anchor under different workspace layouts.

## Build

```bash
cargo build -p flare-grpc-proto
```

After modifying any `proto/*.proto` or any referenced file under `flare-protocol/proto`, rebuilding will trigger `build.rs` to regenerate the code.

## Usage

In another crate's `Cargo.toml`:

```toml
[dependencies]
flare-grpc-proto = "2.2"
```

Reference modules as needed, for example:

```rust
use flare_grpc_proto::message;      // flare.message.v1 service types
use flare_grpc_proto::conversation; // Conversation service (some symbols available in non-wasm builds)
use flare_grpc_proto::Message;      // The common message type re-exported from flare_proto
```

The exact paths are authoritatively defined by the `pub mod` and `pub use` in `src/lib.rs`.

## Generation options

- **Client / Server**: both enabled in `build.rs` (`build_client(true)`, `build_server(true)`).
- **Well-known types**: `compile_well_known_types(false)`, aligned with the `flare-proto` strategy.
- **Serde**: some `flare.media.v1` requests/responses have `Serialize` / `Deserialize` attached in `build.rs`, to facilitate scenarios such as HTTP gateways.

## License

Licensed under the [Apache License 2.0](LICENSE).

---

## Next steps

| What you want to do | Where to go |
|---|---|
| **Get it running in five minutes** | [QUICKSTART](https://github.com/flare-im/flare-core-server/blob/main/QUICKSTART.md) — start the services, hand-sign a token, and call the APIs, **without building your own user system** |
| Integrate your own user system | Implement `TokenValidator` (`CoreJwtTokenValidator` for local signature verification / `HttpHookTokenValidator` to call your own endpoint) |
| Add your own business rules | The 9 extension points of `flare-im-hooks`: PreSend / PostSend / Delivery / Recall / MessageRead / MessageReaction / ConversationLifecycle / ConversationMember / GetConversationParticipants |
| Build a UI | [`@flare-im/vue-ui`](https://www.npmjs.com/package/@flare-im/vue-ui) — 107 components, with a contract consistent across four platforms |
| Report a security issue | [SECURITY.md](.github/SECURITY.md), **please do not open a public issue** |

## When you need an account system and social features

The open-source part is **communication infrastructure**. If what you need is a
ready-made account system, friend relationships, group governance (roles /
join approval / muting), or a moments feed, those live in the commercial modules
— building this layer yourself typically takes months, and it is all repetitive
work unrelated to communication.

Enterprise scenarios additionally have SSO / org structure / audit export /
data residency / SLA support.

Inquiries: `flare1522@163.com`

> For the boundary split and the immutable commitments, see [GOVERNANCE](https://github.com/flare-im/flare-core-server/blob/main/GOVERNANCE.md).
> In short: **what has been open-sourced will not be taken back, and the authentication and hooks contracts will always remain open-source and will never be crippled to force payment.**
