# Opener

[![GitHub Release](https://img.shields.io/github/v/release/dingdayu/opener)](https://github.com/dingdayu/opener/releases/latest)

Opener 是一个基于 Tauri + Vue + TypeScript 开发的桌面应用程序，用于处理自定义协议链接并提供系统集成功能。

## 功能特点

- **自定义协议处理**：注册并处理 `opener://` 协议的链接
- **自动启动**：支持系统启动时自动运行
- **系统通知**：应用启动和操作时发送系统通知
- **自动更新**：内置更新检查和安装功能
- **轻量级界面**：简洁的用户界面，占用资源少

## 安装方法

1. 从 [GitHub Releases](https://github.com/dingdayu/opener/releases/latest) 页面下载最新版本
2. 运行安装程序并按照提示完成安装
3. 安装完成后，应用将自动启动

## 使用指南

### 基本设置

1. **自动启动**：在应用界面中启用或禁用系统启动时自动运行
2. **自定义协议**：应用安装后会自动注册 `opener://` 协议

### 协议使用

您可以通过以下方式使用自定义协议：

```
opener://path?path=E:/A/B.txt&callback=https://abc.com/opener/callback?uuid=abc-def-dadef
```

在网页或其他应用中使用此格式的链接，将自动由 Opener 应用处理。

## 开发指南

### 环境要求

- Node.js 16+
- pnpm 9+
- Rust 工具链

### 本地开发

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm tauri dev

# 构建应用
pnpm tauri build
```

## 注意事项

- 首次启动时，系统可能会请求权限以允许应用发送通知和自动启动
- 在 Windows 系统上，可能需要以管理员身份运行才能正确注册自定义协议
- 应用启动后会自动进入后台运行，可以通过系统托盘图标访问
- 更新功能需要网络连接才能检查和下载新版本

## 许可证

本项目基于 MIT 许可证开源。

## 贡献

欢迎提交 Issue 和 Pull Request 来帮助改进这个项目！
