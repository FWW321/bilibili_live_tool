#!/bin/bash

# 发布脚本 - 用于创建新版本并触发GitHub Actions构建
# 使用方法: ./scripts/release.sh [版本号]

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查是否在git仓库中
if ! git rev-parse --is-inside-work-tree > /dev/null 2>&1; then
    echo -e "${RED}错误: 当前目录不是Git仓库${NC}"
    exit 1
fi

# 检查是否有未提交的更改
if ! git diff-index --quiet HEAD --; then
    echo -e "${RED}错误: 有未提交的更改，请先提交或暂存${NC}"
    git status --short
    exit 1
fi

# 获取当前版本
CURRENT_VERSION=$(grep '^version' Cargo.toml | sed 's/version = "\(.*\)"/\1/')
echo -e "${YELLOW}当前版本: $CURRENT_VERSION${NC}"

# 获取新版本号
if [ -n "$1" ]; then
    NEW_VERSION="$1"
    # 验证版本号格式
    if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo -e "${RED}错误: 版本号格式无效，应为 x.y.z${NC}"
        exit 1
    fi
else
    # 自动增加补丁版本
    IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
    NEW_PATCH=$((PATCH + 1))
    NEW_VERSION="$MAJOR.$MINOR.$NEW_PATCH"
    echo -e "${YELLOW}自动增加版本号: $CURRENT_VERSION → $NEW_VERSION${NC}"
fi

# 确认操作
read -p "是否继续发布版本 $NEW_VERSION? (y/N): " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "取消发布"
    exit 0
fi

# 更新Cargo.toml中的版本号
echo "更新 Cargo.toml 版本号..."
sed -i.bak "s/^version = \".*\"/version = \"$NEW_VERSION\"/" Cargo.toml
rm -f Cargo.toml.bak

# 提交版本更新
git add Cargo.toml
git commit -m "chore: bump version to $NEW_VERSION"

# 创建标签
git tag -a "v$NEW_VERSION" -m "Release version $NEW_VERSION"

# 推送到远程仓库
echo -e "${GREEN}推送到远程仓库...${NC}"
git push origin main
git push origin "v$NEW_VERSION"

echo -e "${GREEN}发布成功！GitHub Actions 将自动构建并创建 Release${NC}"
echo -e "${YELLOW}构建状态: https://github.com/$(git config --get remote.origin.url | sed 's/.*://' | sed 's/\.git$//')/actions${NC}"