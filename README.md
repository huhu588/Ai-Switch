# Open Switch

> OpenCode 配置管理工具 | v1.4.28

一个现代化的桌面应用，用于管理 OpenCode 的 Provider、Model、MCP 服务器、技能和规则配置。

## 功能特性

- **Provider 管理** - 多 Provider 配置，支持 OpenAI、Claude、Gemini 等
- **Model 管理** - 模型列表管理，支持从站点自动获取可用模型
- **MCP 服务器** - MCP 服务器配置与管理
- **技能管理** - 安装和管理 OpenCode 技能
- **规则管理** - 自定义规则配置
- **深链接** - 支持 `openswitch://` 协议一键配置
- **备份恢复** - 配置导入导出功能
- **多语言** - 支持中文和英文界面

## 快速开始

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- npm / pnpm / yarn

### 开发

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

## 配置层级

| 层级 | 路径 | 作用范围 |
|------|------|----------|
| 项目配置 | `./.opencode/opencode.json` | 当前项目 |
| 全局配置 | `~/.config/opencode/opencode.json` | 所有项目 |

> 项目配置优先于全局配置

## 技术栈

- **前端**: Vue 3 + TypeScript + Tailwind CSS + Pinia
- **后端**: Rust + Tauri 2.0
- **构建**: Vite 5

## 发布注意事项

### 版本号同步

发布新版本时，**必须同步更新以下三个文件中的版本号**：

| 文件 | 位置 | 用途 |
|------|------|------|
| `src-tauri/tauri.conf.json` | `version` 字段 | Tauri updater 使用，决定自动更新检测 |
| `src-tauri/Cargo.toml` | `version` 字段 | Rust 编译时使用 |
| `package.json` | `version` 字段 | 前端依赖管理 |

> **警告**: 如果版本号不一致，会导致以下问题：
> - 界面显示的版本与实际版本不符
> - 自动更新检测失败（updater 使用 tauri.conf.json 的版本）
> - 用户无法正常更新到新版本

### 发布流程

1. 同步更新三个文件中的版本号
2. 提交代码：`git commit -m "vX.Y.Z: 更新说明"`
3. 创建 tag：`git tag vX.Y.Z`
4. 推送代码和 tag：`git push origin master && git push origin vX.Y.Z`
5. GitHub Actions 会自动构建并发布 Release

## 文档

- [架构设计](docs/ARCHITECTURE.md)
- [深链接配置](docs/DEEP_LINK.md)
- [自动推断 Provider](docs/AUTO_IMPORT_PROVIDER.md)

## License

MIT
