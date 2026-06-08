# AutoStart Manager

一款 Windows 开机自启动程序管理工具，支持模式化管理，一键启动不同场景所需的应用程序。

## 功能特性

- **模式化管理** — 创建不同的启动模式（如办公模式、娱乐模式、开发模式等）
- **一键启动** — 选择模式后一键启动该模式下的所有程序
- **开机自启** — 支持设置某个模式为默认开机启动模式
- **定时退出** — 启动所有程序后，应用自动退出（默认 5 分钟，可自定义）
- **启动延迟** — 可为每个程序设置独立的启动延迟时间，避免同时启动过多程序
- **静默模式** — 开机时不显示界面，直接在后台启动程序后自动退出

## 技术栈

- **前端**：Vue 3 + TypeScript + Tailwind CSS + Pinia
- **后端**：Rust (Tauri v2)
- **构建产物**：Windows MSI / NSIS 安装包（约 3~5 MB）

## 安装

从 [Releases](https://github.com/jupro2/AutostartManager/releases) 页面下载最新版本的安装包：

- `AutoStartManager_x.x.x_x64-setup.exe` — NSIS 安装程序
- `AutoStartManager_x.x.x_x64_en-US.msi` — MSI 安装程序

## 开发

### 环境要求

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.70
- Windows 10/11

### 本地开发

```bash
# 安装依赖
npm install

# 启动开发模式（带热更新）
npm run tauri dev

# 构建安装包
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/` 目录下。

## 使用说明

1. 安装并打开 AutoStart Manager
2. 点击 **+ 新建模式** 创建一个启动模式（如"办公"、"游戏"）
3. 在模式中点击 **+ 添加程序**，选择需要启动的 `.exe` 文件
4. 为每个程序设置启动延迟（可选）
5. 点击 **▶ 立即启动** 一键启动该模式下的所有程序
6. 如需开机自启，点击模式卡片上的 ⚡ 图标设置为默认启动模式

## 配置文件

配置存储在 `%APPDATA%/AutoStartManager/config.json`，格式如下：

```json
{
  "modes": [
    {
      "id": "uuid",
      "name": "办公模式",
      "icon_name": "briefcase",
      "is_default": true,
      "apps": [
        {
          "id": "uuid",
          "name": "WeChat",
          "path": "C:\\Program Files\\Tencent\\WeChat\\WeChat.exe",
          "args": "",
          "enabled": true,
          "delay_seconds": 5
        }
      ]
    }
  ],
  "exit_delay_minutes": 5,
  "auto_start_enabled": true,
  "silent_launch": true
}
```

## 许可证

MIT License
