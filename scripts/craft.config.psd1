# craft.config.psd1 - per-repo settings for start.ps1 and scripts\release.ps1.
@{
    Name        = 'agtravelcraft'
    Display     = 'AgTravelCraft'
    AppPackage  = 'agtravelcraft-app'
    AppBin      = 'agtravelcraft'
    AppSmoke    = @('--smoke')
    CliPackage  = 'agtravelcraft-mcp'
    CliBin      = 'agtravelcraft-mcp'
    CliSmoke    = @('--db', '{db}', 'demo', '--json')
    McpPackage  = 'agtravelcraft-mcp'
    McpBin      = 'agtravelcraft-mcp'
    McpArgs     = @('mcp')
    McpSmoke    = @('mcp')
    Installer   = $true
    Mcpb        = $true
}
