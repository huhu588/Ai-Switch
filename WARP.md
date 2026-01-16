# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## 项目概述

`opcd` 是一个 Rust CLI 工具，用于管理 OpenCode 的 Provider 和 Model 配置。支持 TUI 交互界面和命令行模式。

## 常用命令

```bash
# 构建
cargo build
cargo build --release

# 运行
cargo run              # 启动 TUI 交互界面
cargo run -- status    # 查看当前配置状态
cargo run -- export opencode  # 导出配置到当前项目

# 测试
cargo test
cargo test -- --nocapture  # 显示测试输出

# 代码检查
cargo clippy
cargo fmt --check
```

## 架构概览

### 模块结构

```
src/
├── main.rs          # 入口，CLI 命令分发
├── cli.rs           # Clap CLI 定义
├── error.rs         # 错误类型定义
├── config/          # 配置管理层
│   ├── manager.rs         # 核心配置管理器，协调各子管理器
│   ├── opencode_manager.rs # OpenCode Provider/Model 配置管理
│   ├── mcp_manager.rs      # MCP 服务器配置管理
│   ├── models.rs           # 数据结构定义 (GlobalConfig, OpenCodeConfig 等)
│   └── detector.rs         # 站点检测功能
├── tui/             # TUI 界面层 (基于 Ratatui)
│   ├── app.rs       # App 状态机核心，管理所有 UI 状态
│   ├── handlers/    # 键盘事件处理 (global.rs)
│   ├── ui/          # UI 渲染
│   │   ├── layout.rs       # 主布局渲染
│   │   └── components/     # UI 组件 (对话框、表单等)
│   ├── types.rs     # TUI 类型定义 (AppTab, InputMode 等)
│   └── theme/       # 主题配置
└── utils/           # 工具函数 (输出格式化)
```

### 核心数据流

1. **ConfigManager** (`config/manager.rs`) 是配置管理的核心入口
   - 管理全局配置 `~/.opcd/config.json`
   - 持有 `OpenCodeConfigManager` 和 `McpConfigManager` 子管理器
   - 处理配置的读写、验证和同步

2. **配置层级**
   - 全局配置: `~/.opcd/` 目录
   - 项目配置: `./.opencode/opencode.json` (当前目录)
   - 项目配置优先于全局配置

3. **TUI 状态管理** (`tui/app.rs`)
   - `App` 结构体维护所有 UI 状态
   - 使用 `InputMode` 区分导航/编辑模式
   - `AppTab` 定义 Tab 页切换 (Providers, MCP, Backup, Status)

### 关键类型

- `OpenCodeConfig`: Provider 和 Model 的配置集合
- `OpenCodeProvider`: 单个 Provider 配置 (name, options, models)
- `McpServer`: MCP 服务器配置

## 开发注意事项

- TUI 使用 Ratatui 框架，事件循环在 `tui/mod.rs`
- 所有键盘事件处理在 `tui/handlers/global.rs`
- 配置文件使用 JSON 格式，序列化使用 serde
- 异步操作使用 tokio 运行时
