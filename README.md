# 智算信息查询系统 - 数据中心事业部

> Tauri 2 + Vue 3 + Rust 桌面应用
> 安装包 < 50M,启动 < 1 秒

## 功能

- 🔬 智算芯片库（28 行产品,各精度算力）
- 🖥️ 服务器库（8 行整机配置,算力 + 功耗）
- 🧠 超节点库（2 行超算规模,功耗）
- 🔍 全局命令面板搜索（Cmd+K / Ctrl+K / `/`）
- 📊 报表 + 列筛选 + 数字范围
- 📦 内置数据,首次启动自动导入

## 开发

```bash
# 前置:Rust 1.77+, Node 18+
npm install
npm run tauri:dev      # 开发模式,带 HMR
npm run tauri:build    # 生产构建
```

## 打包

- `npm run tauri:build` — 当前平台
- `npm run tauri:build -- --target aarch64-apple-darwin` — macOS arm64
- `npm run tauri:build -- --target x86_64-pc-windows-msvc` — Windows

## 自动发布

打 tag 自动构建并发布 GitHub Release:

```bash
git tag v2.0.0
git push origin v2.0.0
```

详细配置见 `.github/workflows/release.yml`。
