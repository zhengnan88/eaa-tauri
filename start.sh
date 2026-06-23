#!/bin/bash
# EAA 启动脚本
cd /Users/zhengnan/Workspace/eaa-tauri

# 杀掉已有的进程
pkill -f "eaa-tauri" 2>/dev/null || true
pkill -f "vite" 2>/dev/null || true

sleep 1

# 启动应用
cargo tauri dev
