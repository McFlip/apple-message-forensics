<#
.SYNOPSIS
    Starts a simple static HTTP web server on an available localhost port and opens the default browser.

.DESCRIPTION
    Gets an available ephemeral TCP port on localhost, binds an HttpListener, serves static files from
    the specified root directory (defaulting to current directory or report folder if present),
    and launches the system default web browser to the server URL.

.PARAMETER RootDirectory
    The root directory to serve files from. Defaults to the current working directory, or 'report' if it exists.

.PARAMETER Port
    Optional specific port to bind. If omitted or 0, an available dynamic port is selected.

.PARAMETER NoBrowser
    If specified, suppresses launching the default web browser.

.EXAMPLE
    ./scripts/serve.ps1
    Serves files from the current directory on an available port and opens the browser.

.EXAMPLE
    ./scripts/serve.ps1 -RootDirectory ./output/report
    Serves files from ./output/report on an available port and opens the browser.
#>
[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$RootDirectory = ".",

    [Parameter()]
    [int]$Port = 0,

    [Parameter()]
    [switch]$NoBrowser
)

$ErrorActionPreference = 'Stop'

# Resolve root directory
$resolvedPath = (Resolve-Path -LiteralPath $RootDirectory).Path
if (-not (Test-Path -LiteralPath $resolvedPath -PathType Container)) {
    Write-Error "The specified root directory does not exist or is not a directory: $RootDirectory"
    exit 1
}

# Determine available port if not specified
if ($Port -le 0) {
    $tcpListener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Loopback, 0)
    $tcpListener.Start()
    $Port = ($tcpListener.LocalEndpoint).Port
    $tcpListener.Stop()
}

$prefix = "http://localhost:$Port/"
$httpListener = [System.Net.HttpListener]::new()
$httpListener.Prefixes.Add($prefix)

try {
    $httpListener.Start()
}
catch {
    Write-Error "Failed to start HTTP listener on ${prefix}: $_"
    exit 1
}

# MIME type mapping dictionary
$mimeTypes = @{
    ".html" = "text/html; charset=utf-8"
    ".htm"  = "text/html; charset=utf-8"
    ".css"  = "text/css; charset=utf-8"
    ".js"   = "application/javascript; charset=utf-8"
    ".mjs"  = "application/javascript; charset=utf-8"
    ".json" = "application/json; charset=utf-8"
    ".xml"  = "application/xml; charset=utf-8"
    ".txt"  = "text/plain; charset=utf-8"
    ".csv"  = "text/csv; charset=utf-8"
    ".png"  = "image/png"
    ".jpg"  = "image/jpeg"
    ".jpeg" = "image/jpeg"
    ".gif"  = "image/gif"
    ".webp" = "image/webp"
    ".svg"  = "image/svg+xml"
    ".ico"  = "image/x-icon"
    ".pdf"  = "application/pdf"
    ".zip"  = "application/zip"
    ".woff" = "font/woff"
    ".woff2"= "font/woff2"
    ".ttf"  = "font/ttf"
    ".otf"  = "font/otf"
    ".mp3"  = "audio/mpeg"
    ".mp4"  = "video/mp4"
    ".webm" = "video/webm"
    ".wav"  = "audio/wav"
}

function Open-DefaultBrowser {
    param([string]$Url)

    try {
        if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
            $psi = [System.Diagnostics.ProcessStartInfo]::new($Url)
            $psi.UseShellExecute = $true
            [System.Diagnostics.Process]::Start($psi) | Out-Null
        }
        elseif ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)) {
            [System.Diagnostics.Process]::Start("open", $Url) | Out-Null
        }
        else {
            [System.Diagnostics.Process]::Start("xdg-open", $Url) | Out-Null
        }
    }
    catch {
        Write-Warning "Could not automatically launch browser: $_"
    }
}

Write-Host "Serving '$resolvedPath' at $prefix"
Write-Host "Press Ctrl+C to stop the server."

if (-not $NoBrowser) {
    Open-DefaultBrowser -Url $prefix
}

# Graceful cleanup on script cancellation / exit
$running = $true

try {
    while ($httpListener.IsListening) {
        $contextAsync = $httpListener.BeginGetContext({
            param($ar)
        }, $null)

        # Wait on handle with timeout so Ctrl+C / interruptions are processed cleanly
        while (-not $contextAsync.AsyncWaitHandle.WaitOne(200)) {
            if (-not $httpListener.IsListening) {
                break
            }
        }

        if (-not $httpListener.IsListening) {
            break
        }

        $context = $httpListener.EndGetContext($contextAsync)
        $request = $context.Request
        $response = $context.Response

        try {
            $rawPath = [System.Uri]::UnescapeDataString($request.Url.AbsolutePath.TrimStart('/'))
            if ([string]::IsNullOrEmpty($rawPath)) {
                $rawPath = "."
            }

            $targetPath = [System.IO.Path]::GetFullPath([System.IO.Path]::Combine($resolvedPath, $rawPath))

            # Directory traversal prevention
            $relative = [System.IO.Path]::GetRelativePath($resolvedPath, $targetPath)
            if ($relative -eq ".." -or $relative.StartsWith(".." + [System.IO.Path]::DirectorySeparatorChar) -or $relative.StartsWith(".." + [System.IO.Path]::AltDirectorySeparatorChar)) {
                $response.StatusCode = 403
                $forbiddenBytes = [System.Text.Encoding]::UTF8.GetBytes("403 Forbidden")
                $response.ContentType = "text/plain; charset=utf-8"
                $response.ContentLength64 = $forbiddenBytes.Length
                $response.OutputStream.Write($forbiddenBytes, 0, $forbiddenBytes.Length)
                $response.OutputStream.Close()
                continue
            }

            # If targeting a directory, check for index.html / index.htm
            if ([System.IO.Directory]::Exists($targetPath)) {
                $indexHtml = [System.IO.Path]::Combine($targetPath, "index.html")
                $indexHtm = [System.IO.Path]::Combine($targetPath, "index.htm")
                if ([System.IO.File]::Exists($indexHtml)) {
                    $targetPath = $indexHtml
                }
                elseif ([System.IO.File]::Exists($indexHtm)) {
                    $targetPath = $indexHtm
                }
            }

            if ([System.IO.File]::Exists($targetPath)) {
                $ext = [System.IO.Path]::GetExtension($targetPath).ToLowerInvariant()
                $contentType = if ($mimeTypes.ContainsKey($ext)) { $mimeTypes[$ext] } else { "application/octet-stream" }

                $fileStream = [System.IO.File]::OpenRead($targetPath)
                try {
                    $response.StatusCode = 200
                    $response.ContentType = $contentType
                    $response.ContentLength64 = $fileStream.Length
                    $fileStream.CopyTo($response.OutputStream)
                }
                finally {
                    $fileStream.Dispose()
                }
            }
            else {
                $response.StatusCode = 404
                $notFoundBytes = [System.Text.Encoding]::UTF8.GetBytes("404 Not Found")
                $response.ContentType = "text/plain; charset=utf-8"
                $response.ContentLength64 = $notFoundBytes.Length
                $response.OutputStream.Write($notFoundBytes, 0, $notFoundBytes.Length)
            }
        }
        catch {
            try {
                $response.StatusCode = 500
                $errBytes = [System.Text.Encoding]::UTF8.GetBytes("500 Internal Server Error: $($_.Exception.Message)")
                $response.ContentType = "text/plain; charset=utf-8"
                $response.ContentLength64 = $errBytes.Length
                $response.OutputStream.Write($errBytes, 0, $errBytes.Length)
            }
            catch {}
        }
        finally {
            try {
                $response.OutputStream.Close()
            }
            catch {}
        }
    }
}
finally {
    if ($httpListener.IsListening) {
        $httpListener.Stop()
    }
    $httpListener.Close()
    Write-Host "`nServer stopped."
}
