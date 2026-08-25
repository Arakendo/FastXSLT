[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$repositoryRoot = Split-Path -Parent $PSScriptRoot
$ignoredSegments = @(
    "${repositoryRoot}\.git\",
    "${repositoryRoot}\target\",
    "${repositoryRoot}\third-party\",
    "${repositoryRoot}\third-party-no-redist\"
)
$failures = [System.Collections.Generic.List[string]]::new()

$markdownFiles = Get-ChildItem -LiteralPath $repositoryRoot -Recurse -File -Filter '*.md' |
    Where-Object {
        $candidate = $_.FullName
        -not ($ignoredSegments | Where-Object { $candidate.StartsWith($_) })
    }

foreach ($file in $markdownFiles) {
    $content = Get-Content -Raw -LiteralPath $file.FullName
    $links = [regex]::Matches($content, '(?<!!)\[[^\]]+\]\((?<target>[^)]+)\)')

    foreach ($link in $links) {
        $target = $link.Groups['target'].Value.Trim().Trim('<', '>')
        if (
            [string]::IsNullOrWhiteSpace($target) -or
            $target.StartsWith('#') -or
            $target -match '^(?i:https?|mailto):'
        ) {
            continue
        }

        $pathPart = $target.Split('#', 2)[0]
        $decodedPath = [System.Uri]::UnescapeDataString($pathPart)
        $resolvedPath = [System.IO.Path]::GetFullPath(
            (Join-Path -Path $file.DirectoryName -ChildPath $decodedPath)
        )

        if (-not (Test-Path -LiteralPath $resolvedPath)) {
            $relativeFile = [System.IO.Path]::GetRelativePath($repositoryRoot, $file.FullName)
            $failures.Add("${relativeFile}: missing link target '${target}'")
        }
    }
}

if ($failures.Count -gt 0) {
    $failures | ForEach-Object { Write-Error $_ }
    throw "Markdown link validation found $($failures.Count) missing target(s)."
}

Write-Host "Checked local links in $($markdownFiles.Count) Markdown files."

