# TXW81x WNB Web UI

简体中文 | [English](README.md)

Taixin Semiconductor TXW81x 系列 SoC WNB（Wireless Network Bridge，无线网络桥）
项目的 Web 管理界面。

本仓库包含一个 Vue 3 单页应用，用于配置和管理 WNB 设备。生产构建会被优化为
单个 HTML 文件，便于嵌入到设备固件中。

## 功能

- 设备登录和认证后的配置流程
- 状态概览和已连接客户端信息
- LAN 网络配置
- 基础和高级 WiFi 配置
- 设备管理和固件升级页面
- 英文和简体中文本地化
- 面向 `192.168.123.1` 设备地址的开发代理

## 技术栈

- Vue 3
- TypeScript
- Pinia
- Vue Router
- Vue I18n
- Vite

## 环境要求

- Node.js 18 或更新版本
- npm
- 如需在开发时联调设备 API，需要一台可通过 `http://192.168.123.1` 访问的
  TXW81x WNB 设备

## 快速开始

安装依赖：

```bash
npm install
```

启动开发服务器：

```bash
npm run dev
```

开发期间，Vite 会将以下设备 API 路径代理到 `http://192.168.123.1`：

- `/post.action`
- `/auth.action`
- `/fwupg.data`

## 构建

创建生产构建：

```bash
npm run build
```

构建流程会先通过 `vue-tsc` 执行 TypeScript 检查，然后使用 Vite 构建项目。
输出文件写入 `dist/`，并通过 `vite-plugin-singlefile` 配置为单文件 HTML 输出。

本地预览生产构建：

```bash
npm run preview
```

## 项目结构

```text
src/
  assets/       静态资源和图标
  components/   共享 Vue 组件
  i18n/         本地化配置
  router/       路由定义和认证守卫
  store/        Pinia 状态管理
  utils/        共享校验器、常量和工具函数
  views/        页面级 Vue 组件
  wnbCom/       WNB 设备 API 通信层
```

## 说明

- 设备 API 使用自定义 HTTP 协议，实现在 `src/wnbCom/` 中。
- 最终 Web UI 包面向受限固件环境，因此需要关注包体大小。
- 当前未配置独立的 lint 或单元测试脚本。

## 许可证

本项目基于 Apache License, Version 2.0 授权。详情见 [LICENSE](LICENSE)。
