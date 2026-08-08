# 智算信息查询系统 - Windows 构建说明

## ⚠️ 为什么不在 Mac 上打 Windows 包

Tauri 在 macOS 上交叉编译 Windows 安装包需要：
- MinGW-w64 工具链（约 300M）
- Wine（运行 NSIS）
- NSIS 安装器

**当前问题**：国内镜像都装不上 MinGW（USTC/清华/阿里云都 404，GitHub release 也被限速到 10K/s），需要 1-2 小时才能装好且容易失败。

**官方推荐**：macOS 跑 macOS，Windows 跑 Windows，跨平台用 CI 自动化。

---

## ✅ Windows 机器构建（30 分钟搞定）

### 1. 准备环境（Windows 机器）

**A. 安装 Rust**（如果还没装）
```powershell
# 打开 PowerShell
winget install Rustlang.Rustup
# 或去 https://rustup.rs/ 下载 rustup-init.exe
```

**B. 安装 Node.js 18+**
```powershell
winget install OpenJS.NodeJS.LTS
```

**C. 安装 Microsoft Visual Studio Build Tools**
```powershell
winget install Microsoft.VisualStudio.2022.BuildTools
# 安装时勾选: "使用 C++ 的桌面开发" workload
# 或直接装 VS Community: winget install Microsoft.VisualStudio.2022.Community
```

**D. 安装 WebView2 Runtime**（Win11 默认有，Win10 需装）
- 微软下载: https://developer.microsoft.com/microsoft-edge/webview2/
- 选 "Evergreen Standalone Installer" 下载装上

### 2. 把源码复制到 Windows 机器

**方式 A**（推荐）：把整个项目 zip 包传过去
- 项目路径：`/Users/silence/Desktop/MiniMAX/智算数据-tauri`
- 打包：`tar -czf zhisuan-tauri-source.tar.gz --exclude=node_modules --exclude=target --exclude=dist --exclude=release 智算数据-tauri/`
- 传到 Windows 后解压

**方式 B**：用 git
```bash
# 在 Mac 上
cd /Users/silence/Desktop/MiniMAX
git init zhisuan-tauri && cd zhisuan-tauri
# (项目代码已经在那,把代码做成 git 仓库,push 到 GitHub/Gitee/自建 git)
```

### 3. 一行命令构建

打开 PowerShell，进入项目目录：
```powershell
cd C:\path\to\智算数据-tauri
npm install            # 5 分钟,下载前端依赖
npm run tauri:build   # 10-15 分钟,编译 Rust + 打包 NSIS
```

### 4. 找产物

构建完成后:
```
src-tauri\target\release\bundle\nsis\
  └── 智算信息查询系统-数据中心事业部-2.0.0-x64.exe   # 约 25M
```

也可以用便携版:
```
src-tauri\target\release\zhisuan-data-app.exe
```

---

## 🎯 预期最终体积

| 平台 | 体积 |
|---|---|
| Windows NSIS 安装包 (.exe) | ~25M |
| Windows 便携版 (单文件) | ~25M |
| Windows 解压版目录 | ~30M |

**完美 < 50M 目标**

---

## 🔧 遇到问题怎么办

### "error: linker `link.exe` not found"
→ 没装 Visual Studio Build Tools，重做步骤 1C

### "error: Microsoft Visual C++ 14.0 or greater is required"
→ 同上

### "WebView2 错误 / 白屏"
→ 没装 WebView2 Runtime，做步骤 1D

### 第一次 cargo build 慢（5-10 分钟）
→ 正常！要下载和编译 ~400 个 Rust crate。第二次就秒开。

### 体积比 macOS 大一点
→ 因为 Windows 没有系统 WebView，要内嵌 WebView2 loader（~5M）
