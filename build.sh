#!/bin/bash

# 小微企业管理助手 - 构建脚本
set -e

echo "===== 开始构建小微企业管理助手 ====="

# 检查依赖
echo ">>> 检查依赖..."
which node || { echo "错误: 需要安装 Node.js"; exit 1; }
which cargo || { echo "错误: 需要安装 Rust"; exit 1; }
which npm || { echo "错误: 需要安装 npm"; exit 1; }

# 安装前端依赖（含 @tauri-apps/cli）
echo ">>> 安装前端依赖..."
npm install

# 构建前端
echo ">>> 构建前端..."
npm run build

# 构建 Tauri 应用
echo ">>> 构建桌面应用..."
case "$(uname -s)" in
    Darwin*)
        echo "目标平台: macOS"
        npx tauri build --target universal-apple-darwin
        ;;
    Linux*)
        echo "目标平台: Linux"
        npx tauri build
        ;;
    *)
        echo "未知平台"
        exit 1
        ;;
esac

echo "===== 构建完成 ====="
echo "安装包位置: src-tauri/target/release/bundle/"

# npm install -D @tauri-apps/cli