# Simprint Server

Simprint 应用的后端服务系统。

## 项目简介

Simprint Server 是 Simprint 桌面应用的后端服务，提供用户认证、数据管理、版本更新等核心功能支持。

## 使用示例

### 通过 `build_docker.py` 构建 Docker 部署包

脚本位置：

```bash
./build_docker.py
```

脚本会执行这些步骤：

- 编译 `simprint-server`
- 编译 `console-gateway`
- 编译 `update-gateway`
- 将三个二进制文件复制到项目根目录
- 默认生成一个 `tar.gz` Docker 部署包

在项目根目录执行：

```bash
python3 build_docker.py
```

如果你本地使用 `uv`，也可以直接执行：

```bash
uv run python build_docker.py
```

常用命令：

```bash
# 默认 release 构建，并生成 tar.gz 部署包
python3 build_docker.py
uv run python build_docker.py

# 构建前清理旧二进制
python3 build_docker.py --clean
uv run python build_docker.py --clean

# 只编译并复制二进制，不生成压缩包
python3 build_docker.py --no-package
uv run python build_docker.py --no-package

# 生成 zip 格式部署包
python3 build_docker.py --format zip
uv run python build_docker.py --format zip

# 使用 dev 模式编译
python3 build_docker.py --dev --no-package
uv run python build_docker.py --dev --no-package
```

构建完成后：

- 二进制文件会出现在当前项目根目录：
  - `./simprint-server`
  - `./console-gateway`
  - `./update-gateway`
- 如果未使用 `--no-package`，还会在项目根目录生成：
  - `simprint-server-docker-*.tar.gz`
  - 或 `simprint-server-docker-*.zip`

### 启动服务

服务需要通过配置文件启动，配置文件路径通过命令行参数指定：

```bash
./simprint-server -f=configs/config.dev.toml
```

### 服务入口

Simprint Server 提供三种服务入口：

- **客户端网关**：为桌面应用提供基础服务接口
- **控制台网关**：为管理后台提供管理功能接口
- **更新网关**：为应用更新提供版本检查接口

> **注意**：详细的使用示例和 API 文档将在后续版本中添加。

## 贡献指南

我们欢迎所有形式的贡献！如果您想为 Simprint Server 项目做出贡献，请遵循以下步骤：

1. **Fork 项目**：在 GitHub 上 Fork 本项目
2. **创建分支**：从 `master` 分支创建新的功能分支
3. **提交更改**：进行代码修改并提交，遵循项目的代码规范
4. **推送分支**：将更改推送到您的 Fork
5. **创建 Pull Request**：向主项目提交 Pull Request

### 代码规范

- 遵循 Rust 代码风格和格式化规范
- 提交前运行 `cargo fmt` 和 `cargo clippy`
- 确保所有测试通过
- 数据库迁移文件命名遵循规范
- 添加适当的文档注释

更多详细信息请参考项目目录下的 `AGENTS.md` 文件。

## 许可证信息

本项目采用私有许可证，未经授权不得使用、复制、修改或分发。

## 故障排除

### 常见问题

**Q: 服务无法启动？**

A: 请检查以下事项：

- 确保配置文件路径正确
- 检查配置文件格式是否正确
- 确保数据库连接配置正确
- 检查端口是否被占用
- 查看服务日志获取详细错误信息

**Q: 数据库连接失败？**

A: 请检查：

- 数据库服务是否正在运行
- 数据库连接字符串是否正确
- 数据库用户权限是否足够
- 网络连接是否正常

**Q: Redis 连接失败？**

A: 请检查：

- Redis 服务是否正在运行
- Redis 连接 URL 是否正确
- Redis 认证信息是否正确

**Q: 对象存储服务异常？**

A: 可能的原因：

- 对象存储服务未启动
- 访问密钥配置错误
- 网络连接问题
- 存储桶不存在或权限不足

**Q: 如何查看服务日志？**

A: 服务日志会输出到控制台，生产环境建议配置日志文件输出。

### 获取帮助

如果以上方法无法解决您的问题，请：

- 查看项目的 Issue 列表，看是否有类似问题
- 创建新的 Issue 描述您的问题
- 联系项目维护者
