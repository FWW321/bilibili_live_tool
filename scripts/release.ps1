# 发布脚本 - PowerShell版本
# 使用方法: .\scripts\release.ps1 [版本号]

# 颜色定义
$RED = "`e[31m"
$GREEN = "`e[32m"
$YELLOW = "`e[33m"
$NC = "`e[0m"  # No Color

function Write-Color($text, $color) {
    Write-Host "$color$text$NC"
}

# 检查是否在git仓库中
function Test-GitRepo {
    try {
        $result = git rev-parse --is-inside-work-tree 2>$null
        return $result -eq "true"
    } catch {
        Write-Color "错误: 当前目录不是Git仓库" $RED
        return $false
    }
}

# 检查是否有未提交的更改
function Test-UncommittedChanges {
    try {
        $status = git status --porcelain 2>$null
        if ([string]::IsNullOrEmpty($status)) {
            return $true
        } else {
            Write-Color "错误: 有未提交的更改，请先提交或暂存" $RED
            Write-Host "未提交的更改:"
            $status -split "`n" | ForEach-Object { Write-Host "  $_" }
            return $false
        }
    } catch {
        Write-Color "错误: 无法检查git状态" $RED
        return $false
    }
}

# 获取当前版本
function Get-CurrentVersion {
    try {
        $cargo = Get-Content Cargo.toml -Raw | ConvertFrom-Toml
        return $cargo.package.version
    } catch {
        Write-Color "错误: 无法读取Cargo.toml中的版本号" $RED
        exit 1
    }
}

# 验证版本号格式
function Test-VersionFormat($version) {
    return $version -match '^\d+\.\d+\.\d+$'
}

# 获取新版本号
function Get-NewVersion($current, $inputVersion) {
    if ($inputVersion) {
        $inputVersion = $inputVersion.Trim()
        if (Test-VersionFormat $inputVersion) {
            return $inputVersion
        } else {
            Write-Color "错误: 版本号格式无效，应为 x.y.z" $RED
            exit 1
        }
    } else {
        # 自动增加补丁版本
        $parts = $current -split '\.'
        $major = $parts[0]
        $minor = $parts[1]
        $patch = [int]$parts[2] + 1
        return "$major.$minor.$patch"
    }
}

# 更新Cargo.toml中的版本号
function Update-CargoVersion($newVersion) {
    try {
        $content = Get-Content Cargo.toml -Raw
        $updated = $content -replace 'version = ".*?"', "version = \"$newVersion\""
        Set-Content -Path Cargo.toml -Value $updated -NoNewline
        Write-Color "已更新Cargo.toml版本号为: $newVersion" $GREEN
    } catch {
        Write-Color "错误: 无法更新Cargo.toml" $RED
        exit 1
    }
}

# 提交版本更新
function Commit-VersionUpdate($newVersion) {
    try {
        git add Cargo.toml
        git commit -m "chore: bump version to $newVersion"
        Write-Color "已提交版本更新" $GREEN
    } catch {
        Write-Color "错误: 无法提交更改" $RED
        exit 1
    }
}

# 创建标签
function Create-Tag($newVersion) {
    try {
        git tag -a "v$newVersion" -m "Release version $newVersion"
        Write-Color "已创建标签: v$newVersion" $GREEN
    } catch {
        Write-Color "错误: 无法创建标签" $RED
        exit 1
    }
}

# 推送到远程仓库
function Push-ToRemote($newVersion) {
    try {
        Write-Color "推送到远程仓库..." $YELLOW
        git push origin main
        git push origin "v$newVersion"
        Write-Color "推送成功！" $GREEN
    } catch {
        Write-Color "错误: 无法推送到远程仓库" $RED
        exit 1
    }
}

# 获取仓库URL
function Get-RepoUrl {
    try {
        $url = git config --get remote.origin.url 2>$null
        if ($url) {
            $url = $url -replace '\.git$', '' -replace '^https://github\.com/', ''
            return $url
        }
    } catch {
        # 忽略错误，使用默认值
    }
    return "username/repository"
}

# 添加ConvertFrom-Toml函数
function ConvertFrom-Toml {
    param($Content)
    
    # 简单的TOML解析，仅用于获取version
    $lines = $Content -split "`n"
    foreach ($line in $lines) {
        if ($line -match '^version\s*=\s*"([^"]+)"') {
            return @{ package = @{ version = $matches[1] } }
        }
    }
    return @{ package = @{ version = "0.1.0" } }
}

# 主程序
Write-Color "哔哩哔哩直播工具 - 发布脚本" $YELLOW
Write-Host "=================================="

# 检查环境
if (-not (Test-GitRepo)) {
    exit 1
}

if (-not (Test-UncommittedChanges)) {
    exit 1
}

# 获取当前版本
$currentVersion = Get-CurrentVersion
Write-Color "当前版本: $currentVersion" $YELLOW

# 获取新版本号
$newVersion = Get-NewVersion $currentVersion $args[0]

if ($args[0]) {
    Write-Color "指定版本号: $newVersion" $YELLOW
} else {
    Write-Color "自动增加版本号: $currentVersion → $newVersion" $YELLOW
}

# 确认操作
$confirm = Read-Host "是否继续发布版本 $newVersion? (y/N)"

if ($confirm -notmatch '^[Yy]$') {
    Write-Host "取消发布"
    exit 0
}

# 执行发布流程
Update-CargoVersion $newVersion
Commit-VersionUpdate $newVersion
Create-Tag $newVersion
Push-ToRemote $newVersion

$repoUrl = Get-RepoUrl
Write-Color "发布成功！GitHub Actions 将自动构建并创建 Release" $GREEN
Write-Color "构建状态: https://github.com/$repoUrl/actions" $YELLOW