# notebooklm-cli

`notebooklm-cli` 是一个独立的 NotebookLM 原生 CLI 与本地控制面服务。

它把 `opencli/src/clis/notebooklm` 的 NotebookLM 业务能力迁到独立仓库，并把原先依赖浏览器页面对象的层替换成 `agent-browser` CLI。

## 当前状态

已完成能力：

- 独立 Rust CLI：`describe`、`execute`、`serve`
- 本地 HTTP 服务（端口 12234）
- MCP 工具暴露（13 个工具）
- Skill catalog（3 个技能）
- 统一 manifest / describe 输出
- 共享密码认证模型
- `agent-browser` 绑定层
- 执行历史记录
- Wave 1-2 命令全部实现（13 个命令）

## 已实现命令

只读命令（Wave 1）：

- `status` - 检测 NotebookLM 页面可用性、登录状态与当前 Google 账号
- `list` - 列出所有 notebook
- `get` - 获取 notebook 元数据（emoji、source 数量、时间戳）
- `summary` - 获取 notebook 摘要
- `source_list` - 列出 notebook 的 source
- `source_get` - 按 ID 或标题获取单个 source

只读命令（Wave 2）：

- `source_fulltext` - 获取 source 全文内容
- `source_guide` - 获取 source 指南摘要与关键词
- `history` - 列出对话历史线程
- `note_list` - 列出 Studio 面板的笔记
- `note_get` - 获取单条笔记内容

写入命令（Wave 2）：

- `note_create` - 在 Studio 面板中创建新笔记
- `source_add_youtube` - 向 notebook 添加 YouTube 视频来源

## 已实现 MCP Tools

- `notebooklm_status`
- `notebooklm_list`
- `notebooklm_get`
- `notebooklm_summary`
- `notebooklm_source_list`
- `notebooklm_source_get`
- `notebooklm_source_fulltext`
- `notebooklm_source_guide`
- `notebooklm_history`
- `notebooklm_note_list`
- `notebooklm_note_get`
- `notebooklm_note_create`
- `notebooklm_source_add_youtube`

## 已实现 Skills

- `research_notebook` — 获取 notebook 概要：摘要 + source 列表 + 对话历史
- `deep_read_source` — 深度阅读 source：指南摘要 + 全文
- `notebook_overview` — 全局概览：列出所有 notebook 并检查状态

## 本地运行

构建：

```bash
cargo build --release
```

查看自描述：

```bash
target/release/notebooklm-cli describe --json
```

执行命令：

```bash
target/release/notebooklm-cli execute list --params '{"cdp_port":"9222"}'
```

启动服务：

```bash
target/release/notebooklm-cli serve
```

覆盖监听地址：

```bash
target/release/notebooklm-cli serve --host 0.0.0.0 --port 12234
```

默认地址：

- API：`http://127.0.0.1:12234`
- Health：`http://127.0.0.1:12234/health`

## 配置

配置文件路径：

- `${HOME}/.config/notebooklm-cli/config.toml`

首次启动会自动：

1. 创建配置目录
2. 探测 `agent-browser` 二进制路径
3. 写入默认配置（权限 0600）
4. 要求先设置 Console 密码

认证模型：

- Console Cookie：`notebooklm_cli_token`
- API Bearer：`Authorization: Bearer <password>`
- MCP Bearer：`Authorization: Bearer <password>`

## HTTP API 概览

公共接口：

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/health` | 健康检查 |
| `GET` | `/api/bootstrap` | 首次运行状态与 CDP 概览 |
| `POST` | `/api/setup/password` | 首次设置密码 |
| `POST` | `/api/login` | 登录（写入 Cookie） |

受保护接口（需认证）：

| 方法 | 路径 | 说明 |
|------|------|------|
| `GET` | `/api/config` | 获取配置 |
| `POST` | `/api/config` | 更新配置 |
| `GET` | `/api/commands` | 命令列表 |
| `GET` | `/api/history` | 执行历史 |
| `GET` | `/api/mcp/tools` | MCP 工具列表 |
| `GET` | `/api/skills` | Skill 列表 |
| `POST` | `/api/execute/{command}` | 执行命令 |
| `GET` | `/api/cdp-ports` | CDP 端口列表 |
| `PUT` | `/api/cdp-ports` | 更新 CDP 端口（持久化） |
| `POST` | `/api/cdp-ports/refresh` | 触发端口重新发现 |
| `GET` | `/api/accounts` | 账号列表 |
| `POST` | `/api/password/change` | 修改密码 |

## 工作原理

所有命令通过 `agent-browser` 控制 Chrome DevTools Protocol（CDP）端口，在浏览器页面内执行 JavaScript，利用已登录的 Google Cookie 调用 NotebookLM 的内部 `batchexecute` RPC 接口（无需额外登录凭证）。

```
notebooklm-cli execute source_list
        │
        ▼
  AgentBrowserClient
        │  spawn agent-browser --cdp 9222
        ▼
  browser page eval(JS)
        │  fetch batchexecute RPC
        ▼
  notebooklm.google.com
```

## 构建发布

使用脚本本地构建：

```bash
# 默认 release
./build.sh

# 交叉编译（需要 cargo-zigbuild）
./build.sh x86_64-unknown-linux-musl
./build.sh aarch64-unknown-linux-musl
```

CI 在推送到 `main` 分支时自动构建 Linux AMD64 / ARM64 静态二进制并发布到 GitHub Releases。
