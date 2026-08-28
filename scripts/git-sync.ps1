<#
.SYNOPSIS
    MusicGlass Git 自动同步脚本（任务书第 56 节）。
.DESCRIPTION
    检查本地修改 -> 运行轻量检查 -> 选择性 add -> commit -> push 到 origin/main。
    严禁 `git add .`：只添加显式指定的源码/文档/脚本，避免把用户音乐、密钥、临时文件提交。
.PARAMETER Message
    Commit 信息。省略时自动生成 "chore: sync <date>"。
.PARAMETER NoPush
    只 commit 不 push。
.EXAMPLE
    pwsh scripts/git-sync.ps1 -Message "feat: add format detector"
#>
param(
    [string]$Message = "",
    [switch]$NoPush
)

$ErrorActionPreference = 'Stop'

# 仓库根目录（脚本位于 <root>/scripts/）
$Root = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $Root

Write-Host "=== MusicGlass git-sync ==="
Write-Host ("Repo: " + $Root)

# 1. 状态检查
$status = git status --porcelain
if (-not $status) {
    Write-Host "No changes to commit."
    exit 0
}
Write-Host "Changed files:"
$status | ForEach-Object { Write-Host ("  " + $_) }

# 2. 冲突/未合并检查
$unmerged = git status --porcelain | Where-Object { $_ -match '^(UU|AA|DD|AU|UA|DU|UD)' }
if ($unmerged) {
    Write-Host "ERROR: 存在未合并冲突，请先解决再同步。" -ForegroundColor Red
    exit 1
}

# 3. 轻量检查（仅当 Rust 已安装时做 cargo check，失败不阻断提交但给出警告）
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if ($cargo) {
    Write-Host "Running cargo check (workspace)..."
    cargo check --workspace 2>&1 | Out-Host
    if ($LASTEXITCODE -ne 0) {
        Write-Host "WARNING: cargo check 失败，提交仍继续（请尽快修复）。" -ForegroundColor Yellow
    }
} else {
    Write-Host "cargo 未安装，跳过 cargo check。"
}

# 4. 选择性 add —— 显式路径，绝不用 `git add .`
#    此处列出应纳入版本控制的目录/文件类别；敏感/大文件已由 .gitignore 排除。
$addPaths = @(
    'apps', 'crates', 'packages', 'docs', 'scripts', 'tests',
    'fixtures', 'resources', 'third_party',
    'README.md', 'README.zh-CN.md', 'LICENSE', 'LICENSE.md',
    'THIRD_PARTY_NOTICES.md', '.gitignore', 'Cargo.toml', 'Cargo.lock',
    'package.json', 'pnpm-lock.yaml', 'tsconfig.json', '.github'
)
foreach ($p in $addPaths) {
    if (Test-Path $p) {
        git add $p
    }
}

# 5. Commit
if (-not $Message) {
    $Message = "chore: sync " + (Get-Date -Format 'yyyy-MM-dd HH:mm')
}
git commit -m $Message
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: commit 失败。" -ForegroundColor Red
    exit 1
}
Write-Host ("Committed: " + $Message)

# 6. Push
if ($NoPush) {
    Write-Host "NoPush 指定，跳过 push。"
    exit 0
}
git push origin HEAD
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: push 失败，请检查远程/网络。" -ForegroundColor Red
    exit 1
}
Write-Host "Pushed to origin/HEAD."
Write-Host "=== done ==="
