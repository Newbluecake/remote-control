# remote-control

跨网络键盘同步工具 — 在一台机器上按下的键会实时转发并在远程机器上模拟，适用于远程协同观影、演示等场景。

通过轻量级 WebSocket 中继服务器转发按键事件，无需串流视频，各端本地播放即可实现同步操作。

## 功能特性

- **全局键盘同步** — 任意应用中的按键都会被捕获并转发到远程端模拟执行
- **即开即用** — 不依赖特定播放器，适用于 mpv、PotPlayer、VLC 等任意软件
- **同步开关** — `Ctrl+Shift+F12` 快捷键随时开启/关闭键盘同步
- **回声抑制** — SyncGuard 机制防止按键回声导致的无限循环
- **自动重连** — 断线后指数退避自动重连，无需手动干预
- **房间系统** — 自动生成 4 位房间码，也可指定房间加入
- **终端聊天** — 同步的同时可在终端直接发送文字消息
- **按键日志** — 缓冲输出捕获到的按键，以 `Key xN` 格式合并显示
- **跨平台** — 支持 Linux、macOS、Windows
- **单一可执行文件** — 一个二进制文件同时充当服务端和客户端

## 快速开始

### 1. 启动中继服务器

在双方都能访问的机器上运行（VPS 或局域网内任一台机器）：

```bash
remote-control serve
```

默认监听 `0.0.0.0:9090`，可通过 `--bind` 自定义。

### 2. 加入房间

**用户 A**（创建房间）：

```bash
remote-control join --server ws://你的服务器:9090 --nickname alice
```

终端会输出自动生成的房间码：

```
--- Room: X7KP | Peers: 1 | Ctrl+Shift+F12 to toggle sync | Type to chat ---
```

**用户 B**（使用房间码加入）：

```bash
remote-control join --server ws://你的服务器:9090 --room X7KP --nickname bob
```

现在双方的键盘操作会实时同步。

### 3. 使用场景示例

双方各自打开同一部电影，加入同一房间后：
- 一方按空格暂停，另一方也会暂停
- 一方按方向键跳转，另一方同步跳转
- 适用于任意播放器，不需要特殊配置

## 平台说明

| 平台 | 权限要求 |
|------|----------|
| **Windows** | 一般不需要管理员权限。如需向管理员窗口发送按键，则需以管理员身份运行 |
| **macOS** | 需要授予「辅助功能」权限（系统设置 → 隐私与安全性 → 辅助功能）。macOS 版压缩包中附带 `sign-mac.sh` 签名脚本，运行后可精确授权 |
| **Linux** | 需要 X11 环境，Wayland 下可能需要额外配置 |

## CLI 参考

```
remote-control <COMMAND>

Commands:
  serve    启动 WebSocket 中继服务器
  join     加入房间并与远程端同步
```

### `serve`

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `-b, --bind` | `0.0.0.0:9090` | 服务器监听地址 |

### `join`

| 参数 | 默认值 | 说明 |
|------|--------|------|
| `-S, --server` | `ws://localhost:9090` | WebSocket 服务器地址 |
| `-r, --room` | *（自动生成）* | 房间码 |
| `-n, --nickname` | `anon` | 显示名称 |
| `--no-sync` | *（默认开启同步）* | 启动时禁用键盘同步 |

## 架构

```
用户 A                             中继服务器                           用户 B
┌────────────┐    WebSocket    ┌──────────────────┐    WebSocket    ┌────────────┐
│ 键盘 → 客户端 ──────────────→  房间转发         ←────────────── 客户端 → 键盘模拟 │
└────────────┘                 └──────────────────┘                 └────────────┘
```

详细设计文档见 [docs/architecture.md](docs/architecture.md)。

## 从源码构建

```bash
git clone https://github.com/Newbluecake/remote-control.git
cd remote-control
cargo build --release
# 二进制文件位于 target/release/remote-control
```

### 交叉编译

```bash
# Windows (需要 mingw 工具链)
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# macOS (需要在 macOS 上编译)
cargo build --release --target aarch64-apple-darwin
```

## 文档

| 文档 | 说明 |
|------|------|
| [快速上手](docs/getting-started.md) | 详细安装指南 |
| [架构设计](docs/architecture.md) | 系统设计与协议规范 |
| [开发指南](docs/development.md) | 参与开发 |
| [部署指南](docs/deployment.md) | 服务器部署 |
| [CHANGELOG](CHANGELOG.md) | 版本历史 |

## 许可证

MIT
