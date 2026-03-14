# 配置文件说明

本服务通过 `-f <配置文件路径>` 指定使用的配置，例如：

```bash
# 主服务（客户端网关）：用户、工作空间、计费、环境、代理等完整 API
cargo run -- -f configs/config.dev.toml

# 更新网关：仅版本检查、维护状态、健康检查等
cargo run -- -f configs/config.update-gateway.dev.toml
```

## 文件说明

| 文件 | 用途 |
|------|------|
| `config.dev.toml` / `config.test.toml` / `config.prod.toml` | 主服务（客户端网关），含数据库、Redis、对象存储、SMTP、工作空间配额等 |
| `config.update-gateway.dev.toml` / `.prod` / `.test` | 更新网关，仅版本与维护相关接口，配置项更少 |

**说明**：控制台网关（console-gateway）已拆分为独立仓库 `simprint-console-server`，此处不再保留 `config.console-gateway.*` 配置。
