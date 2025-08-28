# 自动发布系统指南

这个仓库配置了完整的自动发布系统，当你推送版本标签时，GitHub Actions会自动构建并发布Release。

## 快速开始

### 1. 创建新版本

根据你的环境选择对应的发布脚本：

#### Nushell 用户
```bash
nu scripts/release.nu [可选的版本号]
```

#### PowerShell 用户 (Windows)
```powershell
.\scripts\release.ps1 [可选的版本号]
```

#### Bash 用户 (Linux/macOS)
```bash
./scripts/release.sh [可选的版本号]
```

### 2. 自动流程

运行发布脚本后：
1. 自动更新 `Cargo.toml` 中的版本号
2. 提交版本更新
3. 创建 Git 标签
4. 推送到远程仓库
5. 触发 GitHub Actions 构建

### 3. GitHub Actions 自动构建

GitHub Actions 会：
- 为 Linux、Windows、macOS 构建二进制文件
- 自动创建 GitHub Release
- 上传所有平台的构建产物
- 生成变更日志

## 版本号规则

- 格式：`x.y.z` (主版本.次版本.补丁版本)
- 如果不指定版本号，脚本会自动增加补丁版本
- 示例：当前版本 `0.1.0` → 自动变为 `0.1.1`

## 使用示例

### 自动增加补丁版本
```bash
nu scripts/release.nu
# 或
.\scripts\release.ps1
```

### 指定新版本号
```bash
nu scripts/release.nu 1.2.3
# 或
.\scripts\release.ps1 1.2.3
```

## 构建产物

每次发布会生成以下文件：
- `bilibili_live_tool-linux-amd64` - Linux 可执行文件
- `bilibili_live_tool-windows-amd64.exe` - Windows 可执行文件
- `bilibili_live_tool-macos-amd64` - macOS 可执行文件

## 手动触发

你也可以手动创建标签来触发构建：

```bash
# 创建标签
git tag -a v1.2.3 -m "Release version 1.2.3"

# 推送标签
git push origin v1.2.3
```

## 故障排除

### 常见问题

1. **权限错误**：确保脚本有执行权限
   ```bash
   chmod +x scripts/release.sh  # Linux/macOS
   ```

2. **Git 错误**：确保你在 Git 仓库中且没有未提交的更改

3. **构建失败**：检查 GitHub Actions 日志获取详细信息

### 查看构建状态

构建状态可以在 GitHub 仓库的 Actions 标签页中查看：
`https://github.com/[用户名]/[仓库名]/actions`

## 工作流程文件

自动构建配置在：
`.github/workflows/release.yml`

这个文件定义了：
- 触发条件：推送 `v*` 标签
- 构建矩阵：Linux、Windows、macOS
- 发布配置：自动创建 Release 并上传构建产物