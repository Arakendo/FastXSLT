param(
    [string]$ExtractedTestsPath = (Join-Path $PSScriptRoot '..\.workbench\oasis-xslt10\extracted-full\testsuite\TESTS'),
    [switch]$IncludeCases,
    [switch]$AsJson
)

$ErrorActionPreference = 'Stop'
$root = (Resolve-Path -LiteralPath $ExtractedTestsPath).Path
$catalogPath = Join-Path $root 'catalog.xml'
if (-not (Test-Path -LiteralPath $catalogPath -PathType Leaf)) {
    throw 'Extracted OASIS TESTS directory must contain catalog.xml'
}

[xml]$catalog = Get-Content -Raw -LiteralPath $catalogPath
$records = [System.Collections.Generic.List[object]]::new()

foreach ($group in @($catalog.'test-suite'.'test-catalog')) {
    $majorPath = [string]$group.'major-path'
    $ordinalById = @{}
    foreach ($case in @($group.'test-case')) {
        $id = [string]$case.id
        $ordinalById[$id] = 1 + [int]$ordinalById[$id]
        $scenario = @($case.scenario)[0]
        if ($scenario.operation -ne 'standard') {
            continue
        }
        $caseDirectory = Join-Path (Join-Path $root $majorPath) ([string]$case.'file-path')
        $principal = @{}
        foreach ($input in @($scenario.'input-file')) {
            if ($input.role -in @('principal-data', 'principal-stylesheet')) {
                $principal[[string]$input.role] = $input.InnerText
            }
        }
        foreach ($role in @('principal-data', 'principal-stylesheet')) {
            if (-not $principal.ContainsKey($role)) {
                continue
            }
            $path = Join-Path $caseDirectory $principal[$role]
            if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
                continue
            }
            $text = [Text.Encoding]::Latin1.GetString([IO.File]::ReadAllBytes($path))
            if ($text -notmatch '<!DOCTYPE') {
                continue
            }
            $records.Add([pscustomobject]@{
                identity = "$($group.submitter)/$id#$($ordinalById[$id])"
                role = $role
                path = [IO.Path]::GetRelativePath($root, $path)
                internal_subset = $text -match '<!DOCTYPE[^>]*\['
                external_identifier = $text -match '<!DOCTYPE[^>]*\b(SYSTEM|PUBLIC)\b'
                entity_declaration = $text -match '<!ENTITY\s'
                parameter_entity_declaration = $text -match '<!ENTITY\s+%'
                attribute_list_declaration = $text -match '<!ATTLIST\s'
                id_typing_candidate = $text -match '<!ATTLIST[\s\S]*?\sID\s'
                notation_declaration = $text -match '<!NOTATION\s'
            })
        }
    }
}

$summary = [ordered]@{
    standard_principal_inputs_with_dtd = $records.Count
    source_inputs = @($records | Where-Object role -eq 'principal-data').Count
    stylesheet_inputs = @($records | Where-Object role -eq 'principal-stylesheet').Count
    internal_subsets = @($records | Where-Object internal_subset).Count
    external_identifiers = @($records | Where-Object external_identifier).Count
    entity_declarations = @($records | Where-Object entity_declaration).Count
    parameter_entity_declarations = @($records | Where-Object parameter_entity_declaration).Count
    attribute_list_declarations = @($records | Where-Object attribute_list_declaration).Count
    id_typing_candidates = @($records | Where-Object id_typing_candidate).Count
    notation_declarations = @($records | Where-Object notation_declaration).Count
}

if ($AsJson) {
    [pscustomobject]@{
        summary = [pscustomobject]$summary
        cases = $records
    } | ConvertTo-Json -Depth 4
    exit 0
}

[pscustomobject]$summary | Format-List
if ($IncludeCases) {
    $records | Sort-Object identity, role | Format-Table -AutoSize
}
