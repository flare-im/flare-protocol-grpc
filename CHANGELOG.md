# 变更记录

本文件只记录**已发布到 crates.io 的版本**。版本号遵循
[语义化版本](https://semver.org/lang/zh-CN/)。

## 2.3.0 — 未发布

### 新增

- `AccessGateway.PushCapability(PushCapabilityRequest)` 与 `PushService.PushCapability`
  (`PushCapabilityRequest`/`PushCapabilityResponse`)：插件实时包（宿主能力 `im.realtime.deliver`）
  原样作为 `DataPacket.capability` 下发给所列用户的全部在线连接；只在线、不入库、不做离线推送、至多一次。
  实现 `AccessGateway` 或 `PushService` 的服务端与测试替身需要补上新方法。

- `control_plane.proto`(`flare.control.v1`):控制面 → 数据面的租户/功能开关投影协议
  `TenantProjection { UpsertTenant, UpsertFeatureFlags, GetVersions }`。真源在控制面
  (flare-im-console),Social 与核各实现一份接收端,按 `version` 幂等。整包派生 serde。
- `TenantProjection.UpsertRoleBundle` 与 `RoleGrant`、`RoleBundleProjectionUpsert`:平台级
  角色权限包投影(角色 → 权限码、已吊销会话 jti、员工会话验签 JWKS),供统一网关 admin 分区
  本地授权;只有 Social 接收端实现,核返回 UNIMPLEMENTED。`TenantProjectionUpsert` 新增
  `organization_id`(7),`TenantVersions` 新增 `role_bundle_version`(4)。
- `TenantProjection.UpsertClientPolicy` 与 `ClientPolicyUpsert`、`BuildPolicy`、`BuildRange`:
  应用的接入清单 serial、发布代数与构建策略,Core 与 Social 各落一份;`TenantVersions` 新增
  `client_policy_version`(6)。实现 `TenantProjection` 的服务端与测试替身需要补上新方法。
- `OnlineService.KickTenant`:按租户踢出全部连接(租户停用时由控制面调用)。
- `ConversationManageService.PurgeConversationHistory`:物理删除会话里服务端时间早于
  `before_ms` 的消息、事件、置顶与标记,会话、成员与边界之后的消息保留,重复调用幂等。
  用于「重新加好友时只丢弃旧关系的历史」——整会话硬删会连同接受之后已经发出的消息一起删掉。
  新增 RPC 与消息,不改动已有字段;老服务端对它返回 UNIMPLEMENTED,调用方据此重试而不是
  退回整会话硬删。
  `PurgeConversationHistoryRequest` 另有 `epoch_attribute`(3)与 `before_epoch`(4):
  按消息 attributes 里的纪元清除,带纪元的消息只看纪元、不看时间,没有纪元的消息回落到
  `before_ms`;纪元围栏下 `before_ms` 可以为 0。老服务端忽略这两个字段,按时间边界执行。
- 三个管理员只读服务,供 admin-gateway 以原样转发的 admin 令牌调用,租户只取自令牌:
  - `ConversationAdminService { AdminListConversations, AdminGetConversation, AdminListParticipants }`:
    按租户分页列会话(游标编码 `updated_at` + `conversation_id`,不接收 `user_id`)、
    读会话详情、分页读成员;
  - `OnlineAdminService { AdminListUserDevices, AdminGetUserPresence }`;
  - `CapabilityAdminService { ListTenantCapabilitySwitches, GetTenantRuntime }`:
    `capability_service.proto` 为此引入 `control_plane.proto`,复用 `TenantStatus` 与 `TenantQuota`。

  都是新服务,没有往已有服务里加方法:已有的 `OnlineService`、`CapabilityService`、
  会话读写服务实现不需要改动就能继续编译。
- `PluginRuntimeInspectResponse.protocols`(字段 8):部署除 manifest 操作外提供的平台协议,
  例如数据导出 `flare.plugin.data.v1`;平台只在报告了该协议的部署上启用承诺数据导出的版本。
  新增字段,旧部署不填即视为没有。
- `MessageReadReceiptService { GetMessageReadSummaries, ListMessageReaders }`(会话服务承载):
  群聊里发送者查看自己消息的已读摘要(已读 / 未读 / 总数,一次一批)与已读或未读成员分页。
  按成员的已读水位计数;在席成员超过服务端上限的群回 `available=false`。新服务,已有的
  会话读服务及其测试替身不需要改动。
- `StorageMessageLookupService.LocateMessages`(Storage Reader 承载):按服务端消息 ID 一次
  查一批(至多 50 个)已落库消息所在的会话,不返回内容。供写路径授权一次核对一批 ID,
  例如已读回执点名的消息都要属于本会话。新服务,已有的 `StorageReaderService` 实现与测试
  替身不需要改动。

### 移除

- `sfu_control.proto`（`flare.sfu.control.v1`）与 crate feature `sfu_control`：通话改为托管插件
  `flare.call`，Core 不再经 SFU 控制面契约调用媒体服务。

## 2.1.0 — 未发布

### 新增

- `UpdateConversationUserSettingsRequest` 增加 `is_mention_only`（字段号 7）。
  语义是「只接收提到我的消息」，与 `is_muted` 正交——两者可以同时开启。

  这是**向后兼容的新增**：字段是 `optional`，字段号此前未被占用（旧版止于 6），
  老客户端不发送它、老服务端忽略它，两侧都不受影响。因此升 minor 而非 major。

## 2.0.1 — 2026-08-03

与实现层 1.1.0 对齐的契约层发布。
