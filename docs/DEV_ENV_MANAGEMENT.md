# 编程环境管理功能设计文档

> 版本: v1.7.0（计划） | 作者: Ai Switch Team

## 概述

Ai Switch 计划新增「编程环境管理」功能模块，用于帮助开发者快速检测、安装和切换本地编程环境（Node.js、Java、Python、Rust、Go、.NET 等），提供统一的版本管理入口。

---

## 一、环境检测（自动扫描本地已安装环境）

### 检测策略

通过 Tauri Shell 插件执行系统命令，自动识别已安装的环境及版本管理器。

### 检测项一览

| 环境 | 检测命令 | 版本管理器检测 |
|------|---------|---------------|
| Node.js | `node --version` | `nvm --version` / `fnm --version` |
| Java | `java --version` | `sdk version` (SDKMAN) / `jabba --version` |
| Python | `python --version` / `python3 --version` | `pyenv --version` / `conda --version` |
| Rust | `rustc --version` | `rustup --version` |
| Go | `go version` | `goenv --version` / `gvm version` |
| .NET | `dotnet --version` | 内置多版本管理 |
| PHP | `php --version` | `phpbrew --version` |
| Ruby | `ruby --version` | `rbenv --version` / `rvm --version` |

### 检测结果数据结构

```typescript
interface DevEnvironment {
  name: string            // 环境名称，如 "Node.js"
  installed: boolean       // 是否已安装
  currentVersion: string | null  // 当前版本
  availableVersions: string[]    // 已安装的所有版本（通过版本管理器获取）
  versionManager: {
    name: string | null    // 版本管理器名称，如 "nvm"
    installed: boolean     // 版本管理器是否已安装
    version: string | null // 版本管理器版本
  }
  path: string | null      // 可执行文件路径
  icon: string             // 图标标识
}
```

### Rust 后端检测命令示例

```rust
#[tauri::command]
pub async fn detect_dev_environments() -> Result<Vec<DevEnvironment>, AppError> {
    let mut envs = Vec::new();

    // Node.js 检测
    envs.push(detect_nodejs().await?);
    // Java 检测
    envs.push(detect_java().await?);
    // Python 检测
    envs.push(detect_python().await?);
    // Rust 检测
    envs.push(detect_rust().await?);
    // Go 检测
    envs.push(detect_go().await?);

    Ok(envs)
}
```

---

## 二、各环境版本管理方案

### 1. Node.js（NVM / FNM）

#### 版本管理器推荐

| 管理器 | 平台 | 特点 |
|--------|------|------|
| **nvm-windows** | Windows | Windows 原生支持，最常用 |
| **nvm** | macOS / Linux | 官方 NVM，Shell 脚本实现 |
| **fnm** | 全平台 | Rust 编写，速度快，跨平台 |

#### 安装版本管理器

**Windows (nvm-windows)**
```powershell
# 方式1: 使用 winget 安装
winget install CoreyButler.NVMforWindows

# 方式2: 使用 Chocolatey 安装
choco install nvm

# 方式3: 手动下载安装
# 访问 https://github.com/coreybutler/nvm-windows/releases
```

**macOS / Linux (nvm)**
```bash
# 使用安装脚本
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash

# 或使用 wget
wget -qO- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash
```

**全平台 (fnm) - 推荐**
```bash
# Windows (PowerShell)
winget install Schniz.fnm

# macOS
brew install fnm

# Linux
curl -fsSL https://fnm.vercel.app/install | bash
```

#### 常用操作

```bash
# 查看已安装版本
nvm list                    # Windows
nvm ls                      # macOS/Linux

# 安装指定版本
nvm install 20              # 安装 Node.js 20 最新版
nvm install 18.19.0         # 安装指定版本

# 切换版本
nvm use 20                  # 切换到 Node.js 20
nvm use 18                  # 切换到 Node.js 18

# 设置默认版本
nvm alias default 20        # macOS/Linux
nvm alias default 20        # Windows 无此功能，使用 nvm use 切换

# 查看远程可用版本
nvm ls-remote               # macOS/Linux
nvm list available          # Windows

# 卸载版本
nvm uninstall 16
```

#### 项目级版本锁定

```bash
# 在项目根目录创建 .nvmrc 文件
echo "20" > .nvmrc

# 进入项目目录时自动切换
nvm use    # 读取 .nvmrc 并切换
```

---

### 2. Java（SDKMAN / Jabba）

#### 版本管理器推荐

| 管理器 | 平台 | 特点 |
|--------|------|------|
| **SDKMAN** | macOS / Linux / WSL | 功能最全，支持多种 JDK 发行版 |
| **jabba** | 全平台 | Go 编写，支持 Windows 原生 |
| **scoop** | Windows | Windows 包管理器，可安装多版本 JDK |

#### 安装版本管理器

**SDKMAN（macOS / Linux / WSL）**
```bash
curl -s "https://get.sdkman.io" | bash
source "$HOME/.sdkman/bin/sdkman-init.sh"
sdk version
```

**Windows 原生（Scoop + JDK）**
```powershell
# 安装 Scoop
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
Invoke-RestMethod -Uri https://get.scoop.sh | Invoke-Expression

# 添加 Java bucket
scoop bucket add java

# 安装 JDK
scoop install openjdk21
scoop install openjdk17
scoop install openjdk11
```

#### 常用操作

```bash
# SDKMAN 操作
sdk list java               # 查看所有可用 JDK 版本
sdk install java 21.0.2-tem # 安装指定版本（Temurin 发行版）
sdk install java 17.0.10-tem
sdk use java 21.0.2-tem     # 临时切换
sdk default java 21.0.2-tem # 设置默认版本
sdk current java             # 查看当前版本

# Scoop 操作（Windows）
scoop reset openjdk21        # 切换到 JDK 21
scoop reset openjdk17        # 切换到 JDK 17
java --version               # 验证当前版本
```

#### 常见 JDK 发行版

| 发行版 | 说明 | 适用场景 |
|--------|------|---------|
| Temurin (Eclipse) | 开源免费，社区维护 | 通用开发 |
| Oracle JDK | Oracle 官方，商用需授权 | 企业生产环境 |
| GraalVM | 高性能，支持原生编译 | 微服务、云原生 |
| Amazon Corretto | AWS 优化，免费长期支持 | AWS 环境 |
| Azul Zulu | 认证兼容，免费社区版 | 通用 |

---

### 3. Python（pyenv / Conda / uv）

#### 版本管理器推荐

| 管理器 | 平台 | 特点 |
|--------|------|------|
| **pyenv** | macOS / Linux | 纯 Python 版本管理，轻量 |
| **pyenv-win** | Windows | pyenv 的 Windows 移植版 |
| **Conda (Miniconda)** | 全平台 | 环境+包管理一体化，适合数据科学 |
| **uv** | 全平台 | Rust 编写，极速，新一代工具 |

#### 安装版本管理器

**pyenv（macOS / Linux）**
```bash
# macOS
brew install pyenv

# Linux
curl https://pyenv.run | bash

# 添加到 shell 配置
echo 'export PYENV_ROOT="$HOME/.pyenv"' >> ~/.bashrc
echo 'command -v pyenv >/dev/null || export PATH="$PYENV_ROOT/bin:$PATH"' >> ~/.bashrc
echo 'eval "$(pyenv init -)"' >> ~/.bashrc
```

**pyenv-win（Windows）**
```powershell
# 使用 pip 安装
pip install pyenv-win --target $HOME\.pyenv

# 或使用 PowerShell 脚本
Invoke-WebRequest -UseBasicParsing -Uri "https://raw.githubusercontent.com/pyenv-win/pyenv-win/master/pyenv-win/install-pyenv-win.ps1" -OutFile "./install-pyenv-win.ps1"; &"./install-pyenv-win.ps1"
```

**uv（全平台推荐）**
```bash
# Windows
powershell -ExecutionPolicy ByPass -c "irm https://astral.sh/uv/install.ps1 | iex"

# macOS / Linux
curl -LsSf https://astral.sh/uv/install.sh | sh
```

#### 常用操作

```bash
# pyenv 操作
pyenv install --list         # 查看可安装版本
pyenv install 3.12.2         # 安装指定版本
pyenv install 3.11.8
pyenv versions               # 查看已安装版本
pyenv global 3.12.2          # 设置全局默认版本
pyenv local 3.11.8           # 设置项目版本（创建 .python-version 文件）
pyenv shell 3.12.2           # 当前 Shell 临时切换

# uv 操作（推荐，速度极快）
uv python list               # 查看可用版本
uv python install 3.12       # 安装 Python 3.12
uv python pin 3.12           # 固定项目 Python 版本
uv venv                      # 创建虚拟环境
uv pip install requests      # 安装包（兼容 pip）

# Conda 操作
conda create -n myenv python=3.12  # 创建环境
conda activate myenv               # 激活环境
conda deactivate                    # 退出环境
conda install numpy                 # 安装包
```

#### 虚拟环境管理

```bash
# 标准 venv（推荐用于项目隔离）
python -m venv .venv
source .venv/bin/activate   # macOS/Linux
.venv\Scripts\activate      # Windows

# uv 虚拟环境（更快）
uv venv
source .venv/bin/activate
```

---

### 4. Rust（rustup）

Rust 官方提供了 **rustup**，是 Rust 唯一推荐的版本管理工具。

#### 安装

```bash
# 全平台通用（Windows 会下载 rustup-init.exe）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Windows 也可通过 winget
winget install Rustlang.Rustup
```

#### 常用操作

```bash
# 查看已安装工具链
rustup show
rustup toolchain list

# 安装/切换工具链
rustup install stable        # 安装稳定版
rustup install nightly       # 安装每夜版
rustup install 1.75.0        # 安装指定版本
rustup default stable        # 设置默认工具链
rustup default nightly       # 切换到 nightly

# 项目级工具链（创建 rust-toolchain.toml）
rustup override set nightly  # 为当前目录设置

# 更新
rustup update                # 更新所有工具链
rustup self update           # 更新 rustup 自身

# 组件管理
rustup component add clippy          # 代码检查
rustup component add rustfmt         # 代码格式化
rustup component add rust-analyzer   # IDE 语言服务器

# 编译目标
rustup target add wasm32-unknown-unknown     # WebAssembly
rustup target add aarch64-apple-darwin       # macOS ARM
rustup target add x86_64-pc-windows-msvc     # Windows
```

#### 项目版本锁定

在项目根目录创建 `rust-toolchain.toml`：
```toml
[toolchain]
channel = "1.75.0"
components = ["rustfmt", "clippy", "rust-analyzer"]
targets = ["wasm32-unknown-unknown"]
```

---

### 5. Go（官方多版本 / goenv）

#### 版本管理方案

| 方案 | 说明 |
|------|------|
| **go install（官方推荐）** | Go 1.21+ 内置多版本管理 |
| **goenv** | 类似 pyenv 的第三方版本管理器 |
| **gvm** | Go Version Manager |

#### 安装

```bash
# 官方安装（https://go.dev/dl/）
# Windows
winget install GoLang.Go

# macOS
brew install go

# Linux
sudo snap install go --classic
```

#### 多版本管理

```bash
# 官方方式：安装多个版本
go install golang.org/dl/go1.22.0@latest
go1.22.0 download
go1.22.0 version         # 使用特定版本

# goenv 方式
git clone https://github.com/go-nv/goenv.git ~/.goenv
goenv install 1.22.0
goenv global 1.22.0
goenv local 1.21.5       # 项目级版本
```

---

### 6. .NET（dotnet SDK 多版本）

.NET SDK 原生支持多版本并行安装。

#### 安装

```bash
# Windows
winget install Microsoft.DotNet.SDK.9

# macOS
brew install dotnet-sdk

# Linux
sudo apt install dotnet-sdk-9.0
```

#### 多版本管理

```bash
# 查看已安装版本
dotnet --list-sdks

# 项目级版本锁定（创建 global.json）
dotnet new globaljson --sdk-version 8.0.100

# global.json 内容
{
  "sdk": {
    "version": "8.0.100",
    "rollForward": "latestFeature"
  }
}
```

---

## 三、一键检测脚本

### Windows (PowerShell)

```powershell
# dev-env-check.ps1 - 开发环境一键检测脚本

Write-Host "========== 开发环境检测 ==========" -ForegroundColor Cyan

# Node.js
Write-Host "`n[Node.js]" -ForegroundColor Green
try { $v = node --version 2>$null; Write-Host "  版本: $v" } catch { Write-Host "  未安装" -ForegroundColor Yellow }
try { $v = nvm version 2>$null; Write-Host "  NVM: $v" } catch { Write-Host "  NVM: 未安装" -ForegroundColor Gray }
try { $v = fnm --version 2>$null; Write-Host "  FNM: $v" } catch { Write-Host "  FNM: 未安装" -ForegroundColor Gray }

# Java
Write-Host "`n[Java]" -ForegroundColor Green
try { $v = java --version 2>&1 | Select-Object -First 1; Write-Host "  版本: $v" } catch { Write-Host "  未安装" -ForegroundColor Yellow }

# Python
Write-Host "`n[Python]" -ForegroundColor Green
try { $v = python --version 2>$null; Write-Host "  版本: $v" } catch { Write-Host "  未安装" -ForegroundColor Yellow }
try { $v = uv --version 2>$null; Write-Host "  uv: $v" } catch { Write-Host "  uv: 未安装" -ForegroundColor Gray }

# Rust
Write-Host "`n[Rust]" -ForegroundColor Green
try { $v = rustc --version 2>$null; Write-Host "  版本: $v" } catch { Write-Host "  未安装" -ForegroundColor Yellow }
try { $v = rustup --version 2>$null; Write-Host "  rustup: $v" } catch { Write-Host "  rustup: 未安装" -ForegroundColor Gray }

# Go
Write-Host "`n[Go]" -ForegroundColor Green
try { $v = go version 2>$null; Write-Host "  版本: $v" } catch { Write-Host "  未安装" -ForegroundColor Yellow }

# .NET
Write-Host "`n[.NET]" -ForegroundColor Green
try { $v = dotnet --version 2>$null; Write-Host "  版本: $v" } catch { Write-Host "  未安装" -ForegroundColor Yellow }

Write-Host "`n=====================================" -ForegroundColor Cyan
```

### macOS / Linux (Bash)

```bash
#!/bin/bash
# dev-env-check.sh - 开发环境一键检测脚本

echo "========== 开发环境检测 =========="

check() {
  local name=$1 cmd=$2
  if command -v "$cmd" &>/dev/null; then
    echo "  ✅ $name: $($cmd --version 2>&1 | head -1)"
  else
    echo "  ❌ $name: 未安装"
  fi
}

echo -e "\n[Node.js]"
check "Node.js" "node"
check "NVM" "nvm"
check "FNM" "fnm"

echo -e "\n[Java]"
if command -v java &>/dev/null; then
  echo "  ✅ Java: $(java --version 2>&1 | head -1)"
else
  echo "  ❌ Java: 未安装"
fi
check "SDKMAN" "sdk"

echo -e "\n[Python]"
check "Python" "python3"
check "pyenv" "pyenv"
check "uv" "uv"
check "conda" "conda"

echo -e "\n[Rust]"
check "Rust" "rustc"
check "rustup" "rustup"

echo -e "\n[Go]"
check "Go" "go"

echo -e "\n[.NET]"
check ".NET" "dotnet"

echo "======================================"
```

---

## 四、Ai Switch 集成方案（功能设计）

### 前端界面设计

新增 `DevEnvView.vue` 页面，包含以下区域：

1. **环境概览卡片**：显示所有已检测到的环境，以卡片形式展示当前版本和版本管理器状态
2. **版本列表面板**：选中某个环境后，显示已安装的所有版本，允许一键切换
3. **快速安装面板**：列出未安装的推荐环境及版本管理器，提供一键安装按钮
4. **环境健康检查**：检测常见问题（如 PATH 配置错误、版本冲突等）

### 路由配置

```typescript
{
  path: '/devenv',
  name: 'devenv',
  component: () => import('@/views/DevEnvView.vue')
}
```

### Tauri 后端命令

```rust
// 新增命令列表
commands::detect_dev_environments,    // 检测所有环境
commands::get_env_versions,           // 获取某环境的已安装版本
commands::switch_env_version,         // 切换版本
commands::install_version_manager,    // 安装版本管理器
commands::install_env_version,        // 安装指定版本
commands::get_env_health_check,       // 环境健康检查
```

### 数据流

```
[前端 DevEnvView.vue]
       │
       ▼ invoke('detect_dev_environments')
[Rust 后端 commands/devenv.rs]
       │
       ▼ 执行系统命令（通过 tokio::process::Command）
[系统 Shell]
       │
       ▼ 解析命令输出
[返回 DevEnvironment[] 给前端]
```

---

## 五、推荐最佳实践

### 全平台推荐工具链

| 环境 | 推荐版本管理器 | 理由 |
|------|---------------|------|
| Node.js | **fnm** | Rust 编写，速度快，跨平台一致体验 |
| Java | **SDKMAN** (非Windows) / **Scoop** (Windows) | 生态最全 / Windows 原生体验 |
| Python | **uv** | 新一代工具，极快，涵盖包管理 |
| Rust | **rustup** | 官方唯一推荐，无替代方案 |
| Go | **官方多版本** | Go 1.21+ 内置支持，足够简单 |
| .NET | **SDK 内置** | 原生多版本并行，无需额外工具 |

### 环境变量注意事项

- 多个版本管理器可能修改 `PATH`，注意冲突
- Windows 上注意系统 `PATH` vs 用户 `PATH` 的优先级
- 使用 `.nvmrc`、`.python-version`、`rust-toolchain.toml` 等项目级文件锁定版本
- 团队协作时将版本配置文件纳入版本控制

---

*最后更新: 2026-02-09*
