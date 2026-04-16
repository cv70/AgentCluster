隔离拓展、扩缩容、运维自动化

# AgentCluster — 多 Agent 集群编排
参考 Kubernetes 设计理念，核心映射关系
- K8s Node      → AgentNode（一台设备）
- K8s Pod       → AgentTask（原子任务）
- Control Plane → ClusterController（集群控制面）
- kubelet       → NodeAgent（本地代理）
- etcd          → ClusterStateStore（状态一致性存储）

核心能力：跨节点任务调度 + 记忆同步（增量 + 最终一致性） + 能力路由（手机缺少截屏能力自动路由到 PC）

AgentCluster：面向多实例分布式场景，将多个隔离 Agent 横向连接协同
两者关系：AgentOS 是集群中每个节点的"操作系统内核"，AgentCluster 是连接多个 AgentOS 节点的"集群编排层"。

## AgentCluster 架构设计

### 设计目标

| 目标 | 说明 |
|------|------|
| **异构节点统一管理** | 手机/PC/服务器等不同设备的 Agent 统一纳管 |
| **任务跨节点调度** | 根据节点能力自动分配任务到最优节点 |
| **状态同步与一致性** | 跨节点的记忆、上下文、状态同步 |
| **自愈与容错** | 节点离线时任务自动迁移 |
| **安全通信** | 节点间通信加密，身份认证 |

### Kubernetes 借鉴与映射

AgentCluster 参考 Kubernetes 的设计哲学：

| Kubernetes 概念 | AgentCluster 对应概念 | 说明 |
|----------------|----------------------|------|
| Node | **AgentNode** | 一个运行 AgentOS 的设备 |
| Pod | **AgentTask** | 一个可调度的原子任务单元 |
| Container | **AgentRuntime** | 任务内的隔离执行环境 |
| Control Plane | **ClusterController** | 集群控制面，管理节点与任务 |
| kubelet | **NodeAgent** | 每个节点上的本地代理 |
| kube-proxy | **AgentRouter** | 节点间消息路由 |
| etcd | **ClusterStateStore** | 集群状态一致性存储 |
| Namespace | **AgentGroup** | 多租户/多用户逻辑隔离 |
| Service | **AgentService** | 任务/能力的稳定访问入口 |
| CRD | **SkillDefinition** | 自定义 Skill/能力声明 |

### 整体架构图

```
                        ┌──────────────────────────────────┐
                        │        Control Plane             │
                        │  ┌────────────────────────────┐  │
                        │  │    ClusterController       │  │
                        │  │  ┌──────────┐ ┌─────────┐ │  │
                        │  │  │ 任务调度  │ │ 节点管理 │ │  │
                        │  │  └──────────┘ └─────────┘ │  │
                        │  │  ┌──────────┐ ┌─────────┐ │  │
                        │  │  │ 状态同步  │ │ 健康监控 │ │  │
                        │  │  └──────────┘ └─────────┘ │  │
                        │  └────────────────────────────┘  │
                        │  ┌──────────────────────────────┐ │
                        │  │    ClusterStateStore (etcd)  │ │
                        │  └──────────────────────────────┘ │
                        └────────────┬─────────────────────┘
                                     │ gRPC / HTTPS
              ┌──────────────────────┼─────────────────────┐
              │                      │                     │
   ┌──────────▼──────────┐ ┌─────────▼───────────┐ ┌──────▼──────────────┐
   │    AgentNode: 手机   │ │    AgentNode: PC    │ │  AgentNode: 服务器  │
   │  ┌────────────────┐ │ │ ┌────────────────┐  │ │ ┌────────────────┐  │
   │  │   NodeAgent    │ │ │ │   NodeAgent    │  │ │ │   NodeAgent    │  │
   │  ├────────────────┤ │ │ ├────────────────┤  │ │ ├────────────────┤  │
   │  │   AgentOS      │ │ │ │   AgentOS      │  │ │ │   AgentOS      │  │
   │  │  Runtime       │ │ │ │  Runtime       │  │ │ │  Runtime       │  │
   │  ├────────────────┤ │ │ ├────────────────┤  │ │ ├────────────────┤  │
   │  │ 本地资源        │ │ │ │ 本地资源        │  │ │ │ 云端资源        │  │
   │  │ (低算力/传感器) │ │ │ │ (中算力/文件)  │  │ │ │ (高算力/存储)  │  │
   │  └────────────────┘ │ │ └────────────────┘  │ │ └────────────────┘  │
   └─────────────────────┘ └────────────────────┘ └────────────────────┘
```

### 核心组件详解

#### ClusterController（控制面）

```
ClusterController
├── NodeRegistry          # 节点注册与心跳检测
├── TaskScheduler         # 跨节点任务分配
│   ├── 节点亲和性调度    # 优先调度到用户常用设备
│   ├── 资源感知调度      # 根据节点 CPU/内存/网络分配
│   └── 地理位置调度      # 数据隐私要求时就近处理
├── StateSync             # 跨节点状态同步（记忆/知识/偏好）
├── HealthMonitor         # 节点健康检查与自愈
└── APIServer             # 对外 REST/gRPC 接口
```

#### NodeAgent（数据面，运行在每个节点）

```
NodeAgent
├── HeartbeatReporter     # 定期上报节点状态（资源、在线时长）
├── TaskExecutor          # 接收并执行 ClusterController 分配的任务
├── LocalCache            # 缓存集群元数据（断网可降级运行）
├── SecureChannel         # 与控制面的 mTLS 加密通道
└── AgentOS Bridge        # 与本地 AgentOS 的通信桥接
```

#### AgentNode 节点描述（Node Spec）

```yaml
apiVersion: agentcluster/v1
kind: AgentNode
metadata:
  name: my-macbook
  labels:
    owner: user-001
    device-type: laptop
    location: home
spec:
  capabilities:
    - text-generation
    - code-execution
    - file-management
    - screen-capture          # 设备独有能力
  resources:
    cpu: "8 cores"
    memory: "16Gi"
    storage: "500Gi"
    network: "wifi"
  models:
    - id: claude-sonnet-4-6
      type: remote-api
    - id: llama3-8b
      type: local
  skills:
    - name: browser-use
      version: "1.2.0"
    - name: code-runner
      version: "0.9.0"
  availability:
    schedule: "0 8 * * 1-5"    # 工作日 8 点后在线
    timezone: "Asia/Shanghai"
```

#### AgentTask（任务调度单元）

```yaml
apiVersion: agentcluster/v1
kind: AgentTask
metadata:
  name: daily-report-gen
  namespace: user-001
spec:
  prompt: "汇总今日工作内容生成日报"
  nodeSelector:
    device-type: laptop        # 优先在笔记本执行
  requiredSkills:
    - workos-weekly
    - ku-operator
  resources:
    memoryLimit: "2Gi"
    timeout: "5m"
  contextSync:
    pull: [memory, knowledge]  # 执行前从集群同步最新记忆
    push: [memory]             # 执行后将新记忆推回集群
  fallback:
    nodeSelector:
      device-type: server      # 笔记本离线时回退到服务器
```

### 核心机制

#### 节点发现与注册

```
启动流程：
1. NodeAgent 启动 → 生成节点唯一 ID（基于设备指纹）
2. 向 ClusterController APIServer 发送注册请求（含节点 Spec）
3. Controller 验证身份（证书/Token）→ 注册节点到 etcd
4. NodeAgent 开始定期心跳（默认 30s），上报资源使用率
5. 节点 90s 无心跳 → 标记为 NotReady，触发任务迁移
```

#### 跨节点记忆同步

```
同步策略：
├── 增量同步（默认）：只同步 delta，减少带宽消耗
├── 最终一致性：允许短暂不一致，定期 reconcile
├── 冲突解决：Last-Write-Wins + 向量时钟 (Vector Clock)
└── 离线队列：断网时本地缓存变更，上线后批量同步

隐私分级：
├── PUBLIC：可同步到集群所有节点
├── DEVICE：仅存储在特定设备（如敏感工作文件）
└── LOCAL：永不离开本机（如密钥、私密会话）
```

#### 能力路由（AgentRouter）

当本节点缺少某种能力时，自动路由到有该能力的节点：

```
请求处理流程：
用户请求 → 本地能力检测 → 本地可满足 → 直接执行
                        ↓
                   本地不可满足
                        ↓
              查询 ClusterController
              找到具备该能力的节点
                        ↓
              通过 AgentRouter 转发请求
                        ↓
              目标节点执行并返回结果
```

**示例场景：**
- 用户在手机上说"截屏分析一下我的桌面" → 路由到 PC 节点（有桌面截屏能力）
- 用户要求跑深度学习推理 → 路由到服务器节点（高算力）
- 用户要求查询本地私密文件 → 强制在指定节点本地执行（隐私策略）


## AgentOS 与 AgentCluster 的关系

```
                    单机场景                集群场景
                  ┌──────────┐           ┌──────────────────┐
                  │ AgentOS  │  聚合为    │  AgentCluster    │
                  │ (单节点) │ ─────────► │  (多节点协同)    │
                  └──────────┘           └──────────────────┘

AgentOS 提供：                AgentCluster 扩展：
- 完整的本地执行能力          - 跨设备任务调度
- 持久化记忆管理              - 跨节点记忆同步
- 工具插件系统                - 能力发现与路由
- 安全沙箱隔离                - 统一的集群管理面
```

### 技术选型

| 模块 | 技术选型 | 理由 |
|------|---------|------|
| 控制面通信 | gRPC + protobuf | 高性能、强类型 |
| 状态存储 | etcd | 强一致性，K8s 验证方案 |
| 节点间消息 | NATS / MQTT | 轻量消息总线，适合边缘设备 |
| 安全认证 | mTLS + JWT | 双向认证，业界标准 |
| 服务发现 | 基于 etcd watch | 简单可靠 |
