@echo off
REM 小微企业管理助手 - Windows 构建脚本

echo ===== 开始构建小微企业管理助手 =====

REM 检查依赖
echo ^>^>^> 检查依赖...
where node || echo 错误: 需要安装 Node.js && exit /b 1
where cargo || echo 错误: 需要安装 Rust && exit /b 1
where npm || echo 错误: 需要安装 npm && exit /b 1

REM 安装前端依赖
echo ^>^>^> 安装前端依赖...
call npm install

REM 构建前端
echo ^>^>^> 构建前端...
call npm run build

REM 构建 Tauri 应用
echo ^>^>^> 构建桌面应用...
echo 目标平台: Windows
call cargo tauri build

echo ===== 构建完成 =====
echo 安装包位置: src-tauri\target\release\bundle\
pause