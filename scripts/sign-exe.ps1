#Requires -Version 5.1
<#
.SYNOPSIS
    使用 Certum 云签名证书（SimplySign）为 exe/dll 签名并做时间戳。

.DESCRIPTION
    前提：
      1. 已在 Windows 安装 Certum SimplySign Desktop（ProCertum CardManager）；
      2. 已登录 SimplySign 并挂载虚拟智能卡，证书已出现在当前用户“个人”证书存储
         （Cert:\CurrentUser\My）中；
      3. 已安装 Windows SDK（含 signtool.exe），或通过 -SigntoolPath 指定。

    脚本会按 SHA256 摘要 + RFC3161 时间戳签名，并用 signtool verify /pa 校验。

.PARAMETER Path
    要签名的文件或目录。目录时会递归查找 .exe（加 -IncludeDll 亦包含 .dll）。

.PARAMETER CertificateName
    证书主题中包含的名称（如公司名），用于从证书存储中挑选证书。

.PARAMETER Thumbprint
    证书指纹（推荐，唯一确定）。填了则优先使用。

.PARAMETER TimestampUrl
    时间戳服务地址，默认 Certum：http://time.certum.pl

.PARAMETER SigntoolPath
    signtool.exe 的完整路径；不填则自动查找。

.PARAMETER IncludeDll
    目录模式下同时签名 .dll。

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File .\scripts\sign-exe.ps1 -Path .\src-tauri\target\release\small-business-manager.exe

.EXAMPLE
    .\scripts\sign-exe.ps1 -Path .\src-tauri\target\release\bundle -CertificateName "Your Company"

.EXAMPLE
    .\scripts\sign-exe.ps1 -Path .\dist -Thumbprint "A1B2C3..." -IncludeDll
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$Path,

    [string]$CertificateName,

    [string]$Thumbprint,

    [string]$TimestampUrl = "http://time.certum.pl",

    [string]$SigntoolPath,

    [switch]$IncludeDll
)

$ErrorActionPreference = "Stop"

function Resolve-Signtool {
    param([string]$Provided)
    if ($Provided) {
        if (-not (Test-Path -LiteralPath $Provided)) {
            throw "指定的 signtool 不存在: $Provided"
        }
        return (Resolve-Path -LiteralPath $Provided).Path
    }

    $cmd = Get-Command signtool.exe -ErrorAction SilentlyContinue
    if ($cmd) { return $cmd.Source }

    $roots = @(
        (Join-Path ${env:ProgramFiles(x86)} "Windows Kits\10\bin"),
        (Join-Path $env:ProgramFiles "Windows Kits\10\bin")
    )
    $candidates = @()
    foreach ($root in $roots) {
        if ($root -and (Test-Path -LiteralPath $root)) {
            $candidates += Get-ChildItem -LiteralPath $root -Filter signtool.exe -Recurse -ErrorAction SilentlyContinue |
                Where-Object { $_.FullName -match "\\x64\\signtool\.exe$" } |
                Sort-Object FullName -Descending
        }
    }
    if ($candidates.Count -gt 0) { return $candidates[0].FullName }

    throw "未找到 signtool.exe。请安装 Windows 10/11 SDK，或用 -SigntoolPath 指定路径。"
}

function Get-SigningCertificate {
    param([string]$Thumbprint, [string]$Name)

    $stores = @("Cert:\CurrentUser\My", "Cert:\LocalMachine\My")
    $all = @()
    foreach ($s in $stores) {
        if (Test-Path -LiteralPath $s) {
            $all += @(Get-ChildItem -LiteralPath $s -ErrorAction SilentlyContinue)
        }
    }

    if ($Thumbprint) {
        $tp = ($Thumbprint -replace "\s", "").ToUpperInvariant()
        $match = $all | Where-Object { $_.Thumbprint -eq $tp }
        if (-not $match) {
            throw "证书存储中未找到指纹为 $tp 的证书。请确认 SimplySign 已登录、虚拟卡已挂载。"
        }
        return ($match | Select-Object -First 1)
    }

    $codeSigning = @($all | Where-Object {
        $_.HasPrivateKey -and (
            @($_.EnhancedKeyUsageList | Where-Object {
                $_.FriendlyName -match "Code Signing" -or $_.ObjectId -eq "1.3.6.1.5.5.7.3.3"
            }).Count -gt 0
        )
    })

    if ($Name) {
        $codeSigning = @($codeSigning | Where-Object { $_.Subject -match [regex]::Escape($Name) })
    }

    if ($codeSigning.Count -eq 0) {
        throw "未找到可用的代码签名证书。请确认：1) SimplySign 已启动并登录；2) 证书已在 Cert:\CurrentUser\My；3) 可用 -CertificateName 或 -Thumbprint 指定。"
    }

    return ($codeSigning | Sort-Object NotAfter -Descending | Select-Object -First 1)
}

function Get-TargetFiles {
    param([string]$Path, [switch]$IncludeDll)
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "路径不存在: $Path"
    }
    $item = Get-Item -LiteralPath $Path
    if ($item.PSIsContainer) {
        $patterns = @("*.exe")
        if ($IncludeDll) { $patterns += "*.dll" }
        $files = @()
        foreach ($p in $patterns) {
            $files += @(Get-ChildItem -LiteralPath $item.FullName -Filter $p -Recurse -File -ErrorAction SilentlyContinue)
        }
        return @($files | Sort-Object FullName -Unique)
    }
    return @($item)
}

try {
    $signtool = Resolve-Signtool -Provided $SigntoolPath
    Write-Host "signtool: $signtool" -ForegroundColor DarkGray

    $cert = Get-SigningCertificate -Thumbprint $Thumbprint -Name $CertificateName
    Write-Host "使用证书: $($cert.Subject)" -ForegroundColor Cyan
    Write-Host "  指纹: $($cert.Thumbprint)" -ForegroundColor Cyan
    Write-Host "  有效期至: $($cert.NotAfter)" -ForegroundColor Cyan
    Write-Host "  时间戳: $TimestampUrl" -ForegroundColor DarkGray

    if ($cert.NotAfter -lt (Get-Date)) {
        throw "证书已过期: $($cert.NotAfter)"
    }

    $files = Get-TargetFiles -Path $Path -IncludeDll:$IncludeDll
    if ($files.Count -eq 0) {
        throw "未找到待签名的 exe/dll 文件: $Path"
    }

    $failed = @()
    foreach ($f in $files) {
        Write-Host "`n==> 签名: $($f.FullName)" -ForegroundColor Yellow
        & $signtool sign /sha1 $cert.Thumbprint /fd SHA256 /tr $TimestampUrl /td SHA256 /v $f.FullName
        if ($LASTEXITCODE -ne 0) {
            Write-Host "签名失败: $($f.FullName)" -ForegroundColor Red
            $failed += $f.FullName
            continue
        }

        & $signtool verify /pa /v $f.FullName
        if ($LASTEXITCODE -ne 0) {
            Write-Host "验证失败: $($f.FullName)" -ForegroundColor Red
            $failed += $f.FullName
            continue
        }
        Write-Host "签名并验证成功: $($f.Name)" -ForegroundColor Green
    }

    if ($failed.Count -gt 0) {
        Write-Host "`n以下文件处理失败：" -ForegroundColor Red
        $failed | ForEach-Object { Write-Host "  $_" -ForegroundColor Red }
        exit 1
    }

    Write-Host "`n全部完成，共签名 $($files.Count) 个文件。" -ForegroundColor Green
}
catch {
    Write-Host "`n错误: $($_.Exception.Message)" -ForegroundColor Red
    exit 2
}
