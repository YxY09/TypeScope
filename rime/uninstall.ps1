# 卸载 TypeScope 的小狼毫提交通道插件。
# 自动定位小狼毫用户目录（RimeUserDir）与安装目录（WeaselRoot），清理所有 *.custom.yaml 中的 typescope_commit 配置行与插件文件。

param(
    [switch]$FullRestore,
    [switch]$Deploy
)

$ErrorActionPreference = "Stop"

# 1. 动态获取小狼毫用户目录 RimeUserDir
$User = $null
try {
    $regUser = Get-ItemProperty -Path "HKCU:\Software\Rime\Weasel" -Name "RimeUserDir" -ErrorAction SilentlyContinue
    if ($regUser -and -not [string]::IsNullOrWhiteSpace($regUser.RimeUserDir)) {
        $User = $regUser.RimeUserDir
    }
} catch {}
if (-not $User) {
    $User = Join-Path $env:APPDATA "Rime"
}

$BackupDir = Join-Path $User "typescope-backup"

if ($FullRestore) {
    if (-not (Test-Path $BackupDir)) {
        Write-Host "没有找到备份目录：$BackupDir"
        exit 1
    }
    $backups = Get-ChildItem $BackupDir -Filter "*.custom.yaml.*" -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending
    if (-not $backups) {
        Write-Host "没有找到任何备份文件。"
        exit 1
    }
    $restored = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
    foreach ($b in $backups) {
        $origName = $b.Name -replace '\.\d{8}-\d{6}$',''
        if ($restored.Add($origName)) {
            $dest = Join-Path $User $origName
            Copy-Item $b.FullName $dest -Force
            Write-Host "已从备份恢复：$dest"
        }
    }
} elseif (Test-Path $User) {
    Get-ChildItem -Path $User -Filter "*.custom.yaml" -File -ErrorAction SilentlyContinue | ForEach-Object {
        $raw = Get-Content $_.FullName -Raw -Encoding UTF8
        if ($raw -match "typescope_commit") {
            $lines = Get-Content $_.FullName -Encoding UTF8 | Where-Object { $_ -notmatch "typescope_commit" }
            Set-Content -Path $_.FullName -Value $lines -Encoding UTF8
            Write-Host "已从 $($_.Name) 移除 TypeScope 配置行。"
        }
    }
}

Remove-Item (Join-Path $User "lua\typescope_rime.dll") -ErrorAction SilentlyContinue
Remove-Item (Join-Path $User "lua\typescope_commit.lua") -ErrorAction SilentlyContinue
Write-Host "已删除 lua\typescope_rime.dll 与 lua\typescope_commit.lua。"

# 动态查找 WeaselDeployer.exe
$Deployer = $null
foreach ($rp in @("HKLM:\SOFTWARE\WOW6432Node\Rime\Weasel", "HKLM:\SOFTWARE\Rime\Weasel", "HKCU:\Software\Rime\Weasel")) {
    try {
        $prop = Get-ItemProperty -Path $rp -Name "WeaselRoot" -ErrorAction SilentlyContinue
        if ($prop -and $prop.WeaselRoot) {
            $exe = Join-Path $prop.WeaselRoot "WeaselDeployer.exe"
            if (Test-Path $exe) { $Deployer = $exe; break }
        }
    } catch {}
}

if ($Deploy -and $Deployer) {
    & $Deployer /deploy
    Write-Host "已触发小狼毫重新部署。"
} elseif ($Deployer) {
    Write-Host "请执行重新部署使卸载生效："
    Write-Host "  & '$Deployer' /deploy"
} else {
    Write-Host "请在任务栏右键小狼毫图标点击「重新部署」使卸载生效。"
}
