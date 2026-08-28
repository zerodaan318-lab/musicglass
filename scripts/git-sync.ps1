# git-sync.ps1 — MusicGlass 自动同步脚本

> 用途：代码发生重要修改后，运行检查 → 提交 → push（任务书第 56 节）。
> 重要：**显式 add 指定文件/目录，禁止 `git add .`**，避免把私钥、用户音乐、临时文件提交进去。

param(
    [string]$Message = "chore: sync"
)

$ErrorActionPreference = "Stop"

# 1) 仅添加项目源码与文档（与 .gitignore 配合，排除二进制/密钥/用户文件）
$files = @(
    "Cargo.toml", "Cargo.lock",
    "crates/", "apps/", "packages/", "docs/", "scripts/", "resources/",
    "README.md", "THIRD_PARTY_NOTICES.md", ".gitignore", "LICENSE"
)

foreach ($f in $files) {
    if (Test-Path $f) {
        git add $f
    }
}

# 2) 显示即将提交的内容
git status --short

# 3) 若有变更则提交
$changed = git status --porcelain
if ($changed) {
    git commit -m $Message
    git push
    Write-Host "PUSHED"
} else {
    Write-Host "NOTHING_TO_COMMIT"
}
