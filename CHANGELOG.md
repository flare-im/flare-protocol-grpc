# 变更记录

本文件只记录**已发布到 crates.io 的版本**。版本号遵循
[语义化版本](https://semver.org/lang/zh-CN/)。

## 2.3.0 — 未发布

### 新增

- `control_plane.proto`:`ClientBuildStats.GetConnectionBuilds(GetConnectionBuildsRequest)
  -> ConnectionBuilds`,平台读应用在各接入网关上的（平台，构建）在线连接数。只有核实现。

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
- `online.proto`:`OnlineService.ReportDeviceState(ReportDeviceStateRequest) ->
  ReportDeviceStateResponse`,接入网关把客户端的 `DEVICE_STATE` 上报转给 Online,按会话 ID 更新
  在线记录里的应用状态、用户注意力与角标基线;会话已不存在时 `success=false`。
  `LoginRequest` 新增 `app_state`(13)与 `attention`(14),建连时带上当时的状态,未填按前台处理;
  `DeviceInfo` 新增 `app_state`(12)、`attention`(13)、`state_since`(14)与 `badge`(15)。
  推送路由据此按设备判定新消息的提示归链路还是归推送通道。新增字段,老调用方不填即按前台。
  实现 `OnlineService` 的服务端与测试替身需要补上新方法。
- `PushService.ReportDeviceHandoff(ReportDeviceHandoffRequest) -> ReportDeviceHandoffResponse`:
  一台设备的新消息提示从链路转交给推送通道时,接入网关或 Online 上报一条
  `flare.common.v1.DeviceHandoff`,Push Proxy 校验后发布到 `flare.im.push.handoff`,由 Push Server
  对这台设备补一次提示。只接受这两个内部工作负载的已验证服务身份。
- `PushService.RevokeDeviceBindings(RevokeDeviceBindingsRequest) -> RevokeDeviceBindingsResponse`:
  服务端吊销设备推送绑定,设备被踢下线、被顶替、被租户踢出或身份会话被吊销时由 Online 调用。
  请求带 `tenant_id`、`user_id`、`device_id`(空表示该用户的全部设备)、`reason` 与必填的
  `registered_before`(Unix 毫秒):只吊销登记时间不晚于它的绑定,设备重新登录后登记的新绑定
  不受迟到的吊销影响。响应 `revoked` 是实际吊销的绑定数,0 不算失败,可以重复调用。与客户端凭
  吊销令牌的 `RevokeDevicePushBinding` 相互独立。
  实现 `PushService` 的服务端与测试替身需要补上这两个新方法。
- `ConversationInternalReadService { ListRecentUnreadConversations }`(会话服务承载):Cell 内
  服务以自己的身份按用户读。`ListRecentUnreadConversationsRequest { user_id, since, limit }`
  返回该用户仍在其中、最后一条消息晚于 `since`(Unix 毫秒)、不是他自己发的、且他还有未读的会话
  (`RecentUnreadConversation`:会话 ID、最后一条消息的 seq / ID / 时间 / 发送者、未读数),从新到旧,
  单次至多 20 个。只接受签名的服务身份,租户只取自签名。供设备转入推送通道后补提示用。
  新服务,已有的会话读服务及其测试替身不需要改动。
- `ConversationInternalReadService` 再增加两个方法,供 Push Server 做提示判定:
  `BatchGetUserNotificationSettings(BatchGetUserNotificationSettingsRequest { user_ids })`
  一次读至多 500 个用户的用户级通知设置,响应 `settings` 以 user_id 为键,每个请求的用户都有一项,
  没有设置过的是默认值;`ListNotificationSubscribers(ListNotificationSubscribersRequest
  { conversation_id, page_token, page_size })` 按 user_id 升序分页列出会话里开启了「全部消息通知」、
  仍在席且没有设免打扰或仅 @我 的成员,每页至多 1000 个,整个列表封顶 5000 人,
  `truncated` 表示到了上限且后面还有。同样只接受签名的服务身份,租户只取自签名。
- `UserNotificationSettingsService { GetUserNotificationSettings, UpdateUserNotificationSettings }`
  (会话服务承载):调用者本人的用户级通知设置读与部分更新,只接受网关代签的用户身份,
  请求不带 user_id 与 tenant_id。`UpdateUserNotificationSettingsRequest` 的勿扰时段整体替换或用
  `clear_quiet_hours` 取消,`base_settings_version` 为 0 时不校验版本。客户端经同步编排
  (`Sync.user_notification_settings` / `Sync.get_user_notification_settings`)到达这里。新服务。
- `UpdateConversationUserSettingsRequest` 新增 `optional bool notify_all`(9):成员在大群里显式开启
  「全部消息通知」;返回的 `ConversationUserSettings.notify_all` 带出当前值。按全部字段构造该请求的
  调用方需要补上新字段。
- `RegisterDevicePushTokenRequest` 新增 `app_bundle`(9)、`environment`(10)、`locale`(11)、
  `time_zone`(12)、`app_version`(13)与 `repeated string capabilities`(14)。`app_bundle` 是
  iOS 的 bundle id 或 Android 的 applicationId,三个通道的绑定都必填,APNs 用它作 topic,
  个推的 Android 厂商通知点开时打开这个包里的页面;
  `environment` 只用于 APNs,取 `sandbox` 或 `production`(空按生产);`capabilities` 只收
  `dataMessages`、`serviceExtension`。任一字段变化都是一次新的登记,要换新的 `binding_id`。
  Push Proxy 另在绑定记录里维护最近活跃时间:设备核对自己的绑定时刷新,60 天(可配)没活跃的
  绑定在路由与投递时视为不存在,由投递侧删除。
- `PushService.ReportPushReceipts(ReportPushReceiptsRequest) -> ReportPushReceiptsResponse`:
  设备的到达回执。一批至多 100 条 `PushReceipt`(会话、消息、seq、合并键、处置、通道
  `apns`/`fcm`/`getui`、来路 `extension`/`app`、设备时钟的收到时刻),`app_version` 至多 32 个
  `[0-9A-Za-z._+-]` 字符,其余记为 `other`。只接受网关代签的用户身份,租户与用户必须等于签名;
  认不出的条目跳过,响应 `accepted` 是记下的条数。回执只进指标与推送决策链,不影响投递。
  实现 `PushService` 的服务端与测试替身需要补上新方法。
- `PushAdminService { AdminQueryPushTrace }`(Push Proxy 承载):管理员按用户查推送决策链,
  `AdminQueryPushTraceRequest { user_id, optional message_id, limit }`,给了 `message_id` 只查这一条,
  否则查最近 `limit` 条(缺省 20,至多 50)。返回 `PushTraceView`(键、会话、开始时间与按时间排列的
  `PushTraceEntryView`:时刻、环节、设备与字段)。只认 admin-gateway 原样转发的 Staff 上下文
  (`core:users:read`),租户取自令牌。新服务,已有的 `PushService` 实现与测试替身不需要改动。

### 移除

- `capability_service.proto`：Hook 点只留 `message.pre_send`。`HookPoint` 删掉其余 16 个取值；删掉
  `PostSendHook*`、`DeliveryHook*`、`RecallHook*`、已读、表态、会话、成员、推送、用户、在线状态和
  `CustomHook*` 这些请求与事件，以及 `HookGroup`、`ExecutionMode`、`HookErrorPolicy`、
  `LifecycleEventType`、`ConversationMemberChangeType`。事后观察归事件。`HookPlugin.Call`、
  `PreSendHookRequest`、`PreSendHookResponse` 不变。
- `capability_service.proto`：Hook 管理消息收敛到「一个点位 + 一个目标」。删 `HookPoint`、`HookRetryPolicy`，
  删 `hook_points`、`retry_policy`、`config_extensions`、`HookTransport.target`，`HookSelector` 只留
  `tenants`、`conversation_types`、`message_types`；`priority` 改名 `order`，新增 `required`，更新请求里
  这两个是 `optional`。`tenant_id` 空是平台 Hook。新增 `ListHookPoints*`、`ListHookProfiles*`、
  `TestHookConfig*`。
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
