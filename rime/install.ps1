# 将 TypeScope 的上屏提交通道接入小狼毫（Weasel / Rime）。
# 特性：
# 1. 自动从注册表读取小狼毫的用户配置目录（RimeUserDir）与安装目录（WeaselRoot），不写死路径。
# 2. 自动兼容源码编译目录（target\release\）与独立发布包目录。
# 3. 自动识别用户当前正在使用的输入方案（从 user.yaml 及现有的 *.custom.yaml 探测，兼容薄荷拼音 rime_mint、雾凇拼音 rime_ice、朙月拼音 luna_pinyin、小鹤双拼等所有方案）。
# 4. 修改前自动备份原配置文件到 typescope-backup\。

param(
    [string]$Schema = "",
    [switch]$Deploy
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

# 1. 查找 typescope_rime.dll（兼容源码构建与发布包）
$DllCandidates = @(
    (Join-Path $PSScriptRoot "typescope_rime.dll"),
    (Join-Path $Root "typescope_rime.dll"),
    (Join-Path $Root "target\release\typescope_rime.dll"),
    (Join-Path $Root "target\debug\typescope_rime.dll")
)
$Dll = $DllCandidates | Where-Object { Test-Path $_ } | Select-Object -First 1
if (-not $Dll) {
    Write-Host "找不到 typescope_rime.dll。请先在项目根目录运行：cargo build --release --workspace"
    exit 1
}

$Lua = Join-Path $PSScriptRoot "typescope_commit.lua"
if (-not (Test-Path $Lua)) {
    Write-Host "找不到 $Lua"
    exit 1
}

# 2. 动态获取小狼毫用户目录 RimeUserDir
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
if (-not (Test-Path $User)) {
    Write-Host "未找到小狼毫用户目录：$User（请确认已安装并初始化小狼毫输入法）"
    exit 1
}

# 3. 动态查找 WeaselDeployer.exe
function Find-WeaselDeployer {
    $regPaths = @(
        "HKLM:\SOFTWARE\WOW6432Node\Rime\Weasel",
        "HKLM:\SOFTWARE\Rime\Weasel",
        "HKCU:\Software\Rime\Weasel"
    )
    foreach ($rp in $regPaths) {
        try {
            $prop = Get-ItemProperty -Path $rp -Name "WeaselRoot" -ErrorAction SilentlyContinue
            if ($prop -and $prop.WeaselRoot) {
                $exe = Join-Path $prop.WeaselRoot "WeaselDeployer.exe"
                if (Test-Path $exe) { return $exe }
            }
        } catch {}
    }
    $searchRoots = @("$env:ProgramFiles\Rime", "${env:ProgramFiles(x86)}\Rime")
    foreach ($sr in $searchRoots) {
        if (Test-Path $sr) {
            $found = Get-ChildItem -Path $sr -Filter "WeaselDeployer.exe" -Recurse -ErrorAction SilentlyContinue |
                Sort-Object FullName -Descending |
                Select-Object -First 1
            if ($found) { return $found.FullName }
        }
    }
    return $null
}

$Deployer = Find-WeaselDeployer

# 4. 自动探测需要挂载的输入方案 (*.custom.yaml)
$GlobalConfigs = @("default", "weasel", "squirrel", "ibus_rime", "fcitx_rime")
$TargetSchemas = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)

if (-not [string]::IsNullOrWhiteSpace($Schema)) {
    $cleanName = $Schema -replace '\.custom\.yaml$','' -replace '\.schema\.yaml$',''
    [void]$TargetSchemas.Add($cleanName)
} else {
    # 从 user.yaml 读取最近使用的方案
    $userYaml = Join-Path $User "user.yaml"
    if (Test-Path $userYaml) {
        $uContent = Get-Content $userYaml -Raw -Encoding UTF8
        if ($uContent -match 'previously_selected_schema:\s*([A-Za-z0-9_\-]+)') {
            $sel = $Matches[1]
            if ($GlobalConfigs -notcontains $sel) {
                [void]$TargetSchemas.Add($sel)
            }
        }
    }
    # 扫描已存在的方案级 *.custom.yaml
    Get-ChildItem -Path $User -Filter "*.custom.yaml" -File -ErrorAction SilentlyContinue | ForEach-Object {
        $base = $_.Name -replace '\.custom\.yaml$',''
        if ($GlobalConfigs -notcontains $base) {
            [void]$TargetSchemas.Add($base)
        }
    }
    # 若还没有任何方案级 custom.yaml，则扫描目录下的 *.schema.yaml
    if ($TargetSchemas.Count -eq 0) {
        Get-ChildItem -Path $User -Filter "*.schema.yaml" -File -ErrorAction SilentlyContinue | ForEach-Object {
            $base = $_.Name -replace '\.schema\.yaml$',''
            if ($GlobalConfigs -notcontains $base) {
                [void]$TargetSchemas.Add($base)
            }
        }
    }
}

if ($TargetSchemas.Count -eq 0) {
    Write-Host "未能自动识别出小狼毫输入方案，请通过参数指定方案名，例如：.\rime\install.ps1 -Schema rime_ice"
    exit 1
}

$LuaDir = Join-Path $User "lua"
$BackupDir = Join-Path $User "typescope-backup"
New-Item -ItemType Directory -Force -Path $BackupDir, $LuaDir | Out-Null

# 复制 DLL 与 Lua 脚本
Copy-Item $Dll (Join-Path $LuaDir "typescope_rime.dll") -Force
Copy-Item $Lua (Join-Path $LuaDir "typescope_commit.lua") -Force

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$line = '  "engine/processors/@before 0": lua_processor@*typescope_commit'
$patchedFiles = @()

foreach ($sName in $TargetSchemas) {
    $customFile = Join-Path $User "$sName.custom.yaml"
    if (Test-Path $customFile) {
        $backup = Join-Path $BackupDir "$sName.custom.yaml.$stamp"
        Copy-Item $customFile $backup -Force
        $text = Get-Content $customFile -Raw -Encoding UTF8
        if ($text -notmatch "typescope_commit") {
            if ($text -notmatch "(?m)^patch\s*:") {
                $text = $text.TrimEnd() + "`r`npatch:`r`n" + $line + "`r`n"
            } else {
                if (-not $text.EndsWith("`n")) { $text += "`r`n" }
                $text += $line + "`r`n"
            }
            Set-Content -Path $customFile -Value $text -Encoding UTF8
        }
    } else {
        $newContent = "patch:`r`n$line`r`n"
        Set-Content -Path $customFile -Value $newContent -Encoding UTF8
    }
    $patchedFiles += "$sName.custom.yaml"
}

Write-Host "已定位小狼毫用户目录：$User"
Write-Host "已复制 typescope_rime.dll 与 typescope_commit.lua 到：$LuaDir"
Write-Host "已配置方案文件：$($patchedFiles -join ', ')（原文件已备份至 $BackupDir）"

if ($Deploy -and $Deployer) {
    Write-Host "正在调用小狼毫重新部署：$Deployer /deploy"
    & $Deployer /deploy
    Write-Host "小狼毫重新部署完成。"
} elseif ($Deployer) {
    Write-Host "请运行以下命令完成小狼毫重新部署（或在任务栏右键小狼毫图标点击「重新部署」）："
    Write-Host "  & '$Deployer' /deploy"
} else {
    Write-Host "请在任务栏右键小狼毫输入法图标，点击「重新部署」使其生效。"
}
