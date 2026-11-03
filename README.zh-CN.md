# flare-grpc-proto

[English](README.md) · 中文

> ## ℹ 这是通信基础设施，不是开箱即用的 IM 产品
>
> 说在前面，免得你 clone 完才发现登不上去：**开源部分不含账号体系**
> （没有注册登录、好友关系、群角色/审批/禁言、朋友圈）。
>
> 但它自带完整且可插拔的鉴权契约，两条路都在开源侧：
>
> - **`CoreJwtTokenValidator`** —— 本地验 JWT。手签一个 token 就能跑起来做
>   demo / POC，**不需要任何用户体系**。
> - **`HttpHookTokenValidator`** —— 把 token POST 到你自己的接口，
>   **这是接入自有用户体系的入口**。
>
> 业务规则同理：`flare-core-server/crates/flare-im-hooks` 提供 9 个扩展点
> （PreSend / PostSend / Delivery / Recall / MessageRead / MessageReaction /
> ConversationLifecycle / ConversationMember / GetConversationParticipants）。
>
> 要上生产，你需要自行实现用户体系并按上述契约接入 —— 与 Sendbird /
> Twilio Conversations 的「自带身份」模型一致，区别是 Flare 可自托管、
> 协议与核心可审计。
>
> 边界详情见 [GOVERNANCE.md](.github/GOVERNANCE.md)。


Flare IM 的 **gRPC 服务层 Protocol Buffers** 与 **Rust 生成代码**（`tonic` + `prost`）。业务服务 crate 通过依赖本库获得各服务的 client / server 类型及消息定义。

## 职责边界

| 组件 | 说明 |
|------|------|
| **`flare-proto`** | 域模型与公共类型（如 `flare.common.v1`、消息体 `Message` 等） |
| **本 crate** | 各 gRPC 服务的 `.proto` 与编译产物；通过 `extern_path` 引用 `flare_proto` 中的公共包，避免重复生成 |

## 目录结构

```text
flare-grpc-proto/
├── proto/           # 服务定义（*.proto）
├── build.rs         # tonic-prost-build 编译入口
├── src/lib.rs       # include_proto! 与便捷 re-export
├── Cargo.toml
└── README.md
```

## 本仓库中的服务 Proto

`build.rs` 当前编译的文件（顺序与依赖解析相关）：

- `access_gateway.proto`
- `conversation_service.proto`
- `capability_service.proto`（包名 `flare.capability.v1`：`CapabilityService`、`HookExtension`、`HookService`）
- `media_service.proto`
- `message_service.proto`
- `online.proto`
- `push_service.proto`
- `router.proto`
- `storage_service.proto`
- `sync_service.proto`

基础类型的 `import` 由 **`flare-protocol/proto`** 提供；编译时 include 路径为「基础目录优先，再本目录」，以减少同名 import 歧义。

## 依赖

- **Rust**：见根目录 `Cargo.toml` 中 `rust-version`（当前为 **1.94+**，Edition **2024**）。
- **crate**：`flare-proto`、`prost`、`tonic`、`serde` 等，详见 `Cargo.toml`。

> `flare-proto` 的 path 以 **`Cargo.toml`** 为准；在不同工作区布局下可能使用相对路径锚定。

## 构建

```bash
cargo build -p flare-grpc-proto
```

修改任意 `proto/*.proto` 或 `flare-protocol/proto` 下被引用的文件后，重新构建即可触发 `build.rs` 重新生成代码。

## 使用方式

在其它 crate 的 `Cargo.toml` 中：

```toml
[dependencies]
flare-grpc-proto = "2.2"
```

按需引用模块，例如：

```rust
use flare_grpc_proto::message;      // flare.message.v1 服务类型
use flare_grpc_proto::conversation; // 会话服务（非 wasm 时部分符号可用）
use flare_grpc_proto::Message;      // 从 flare_proto 再导出的公共消息类型
```

具体路径以 `src/lib.rs` 中的 `pub mod` 与 `pub use` 为准。

## 生成选项说明

- **Client / Server**：均在 `build.rs` 中开启（`build_client(true)`, `build_server(true)`）。
- **Well-known types**：`compile_well_known_types(false)`，与 `flare-proto` 策略对齐。
- **Serde**：部分 `flare.media.v1` 请求/响应在 `build.rs` 中附加 `Serialize` / `Deserialize`，便于 HTTP 网关等场景。

## 许可

依据 [Apache License 2.0](LICENSE) 授权。

---

## 下一步

| 想做什么 | 去哪里 |
|---|---|
| **五分钟跑起来** | [QUICKSTART](https://github.com/flare-im/flare-core-server/blob/main/QUICKSTART.md) —— 起服务、手签 token、调通接口，**不需要自建用户体系** |
| 接入自己的用户系统 | 实现 `TokenValidator`（`CoreJwtTokenValidator` 本地验签 / `HttpHookTokenValidator` 调你的接口） |
| 加自己的业务规则 | `flare-im-hooks` 的 9 个扩展点：PreSend / PostSend / Delivery / Recall / MessageRead / MessageReaction / ConversationLifecycle / ConversationMember / GetConversationParticipants |
| 做界面 | [`@flare-im/vue-ui`](https://www.npmjs.com/package/@flare-im/vue-ui) —— 107 个组件，四端一致的契约 |
| 报安全问题 | [SECURITY.md](.github/SECURITY.md)，**请勿开公开 issue** |

## 需要账号体系与社交能力时

开源部分是**通信基础设施**。如果你需要的是现成的账号、好友关系、群治理（角色 / 入群审批 / 禁言）、朋友圈，
这些在商业模块里 —— 自研这一层通常要数月，且都是与通信无关的重复劳动。

企业场景另有 SSO / 组织架构 / 审计导出 / 数据驻留 / SLA 支持。

咨询：`flare1522@163.com`

> 边界划分与不变承诺见 [GOVERNANCE](https://github.com/flare-im/flare-core-server/blob/main/GOVERNANCE.md)。
> 简言之：**已开源的不会被收回，鉴权与 hooks 契约永远开源、不会为逼迫付费而阉割。**
