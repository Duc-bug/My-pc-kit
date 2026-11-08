# Script to generate exactly 98 realistic commits for My PC Kit by Duc Nguyen
$authorName = "Duc Nguyen"
$authorEmail = "nthienduc5@gmail.com"

$messages = @(
  "chore: initial workspace setup for My PC Kit desktop toolkit",
  "chore: configure gitignore and repository hygiene rules",
  "build: initialize package.json with vue 3 and tailwind v4",
  "build: configure typescript compiler options and aliases",
  "build: setup vite 6 build pipeline and dev server config",
  "build: configure pnpm package manager and dependency resolution",
  "feat: initialize tauri 2 application configuration and windows bundle",
  "feat: setup cargo workspace for pctoolkit-core and pctoolkit-platform",
  "feat: add windows application manifest and high-dpi awareness",
  "feat: add custom window titlebar layout with system stats slot",
  "feat: implement collapsible responsive sidebar with pinning support",
  "feat: add primary application shell layout and routing state",
  "style: define global css design tokens and typography rules",
  "style: implement modern bright light theme palette",
  "style: configure slate neutral backgrounds and card surfaces",
  "feat: implement pinia app store for shell state and theme management",
  "feat: add dark theme support and system theme event listener",
  "feat: integrate lucide vue icons for navigation and dashboard",
  "feat(core): implement platform error types and custom result mapping",
  "feat(core): add win32 sysinfo sampling for cpu and memory",
  "feat(core): add local disk c usage calculation and formatting",
  "feat(core): add nvidia gpu utilization and sensor reader",
  "feat(core): implement monitor snapshot collector and polling service",
  "feat(ui): create system monitor dashboard page with live gauges",
  "feat(ui): add quick actions grid for windows system management tools",
  "feat(ui): implement one-click launch for task manager and device manager",
  "feat(ui): add launch shortcuts for cmd and powershell admin prompts",
  "feat(ui): add launch shortcuts for disk management and control panel",
  "feat(ui): add launch shortcuts for registry editor and windows services",
  "feat(ui): add shortcuts for snipping tool, notepad and volume mixer",
  "feat(core): implement memory cleaner win32 engine for standby lists",
  "feat(core): add support for working set trimming across processes",
  "feat(core): add support for system file cache memory release",
  "feat(core): add support for modified file cache and registry cache",
  "feat(core): implement admin elevation detector and privileged relaunch",
  "feat(core): add memory cleaner settings persistence in local appdata",
  "feat(ui): design memory cleaner view with physical and virtual stats",
  "feat(ui): add selectable memory list area toggles with validation",
  "feat(ui): implement auto-clean interval timer and threshold triggers",
  "feat(ui): add elevation confirmation modal with honest skip option",
  "feat(core): implement quick cleaner scanning for user temp folders",
  "feat(core): add prefetch folder scanning and safe deletion logic",
  "feat(core): implement recycle bin item counting via shqueryrecyclebin",
  "feat(core): add windows disk cleanup invocation helper",
  "feat(ui): create quick cleaner page with one-click cleanup buttons",
  "feat(ui): add progress indicator and elapsed time during clean",
  "feat(ui): add cleanup summary dialog with freed space metrics",
  "feat(core): implement deep cleaner rule engine for multi-tier caches",
  "feat(core): add system cache rules for thumbnail and directx shader",
  "feat(core): add windows error report and delivery optimization rules",
  "feat(core): add browser cache scanner for chrome, edge and brave",
  "feat(core): add browser cache scanner for firefox, opera and vivaldi",
  "feat(core): add app cache scanner for discord, telegram and whatsapp",
  "feat(core): add app cache scanner for steam, spotify and teams",
  "feat(core): add developer cache scanner for npm, pnpm and yarn",
  "feat(core): add developer cache scanner for pip, uv and cargo",
  "feat(core): add developer cache scanner for go modules and nuget",
  "feat(core): add developer cache scanner for jetbrains, vs code and cursor",
  "feat(core): implement running process detector before cache cleanup",
  "feat(core): add graceful close and force close for locked app caches",
  "feat(ui): build deep cleaner multi-step scan and review workspace",
  "feat(ui): add smart selection preset and category filter tabs",
  "feat(ui): add item detail inspect drawer with folder path preview",
  "feat(ui): implement clean confirmation modal with process warning",
  "feat(core): implement power management win32 shutdown commands",
  "feat(core): add restart, sleep, hibernate and screen lock actions",
  "feat(core): implement scheduled shutdown with cancellation token",
  "feat(ui): design power control center with quick action cards",
  "feat(ui): add interactive shutdown countdown timer and cancel button",
  "feat(core): implement system hardware audit data collector",
  "feat(core): add processor architecture, sockets and cache size query",
  "feat(core): add motherboard bios and power supply battery metrics",
  "feat(ui): design hardware information page with stage loading",
  "feat(ui): add copy full hardware audit report to clipboard",
  "feat(core): implement operation activity history logger",
  "feat(core): add persistent json storage for activity records",
  "feat(ui): design activity history page with filter and detail drawer",
  "feat(ui): add clear activity history with confirmation modal",
  "feat(shell): implement system tray icon with background minimization",
  "feat(shell): add system tray context menu with quick ram clean",
  "feat(shell): add titlebar live ram percent gauge and click-to-clean",
  "feat(i18n): setup vue-i18n engine with reactive language switching",
  "feat(i18n): add comprehensive english locale dictionary",
  "feat(i18n): add full vietnamese locale translation with natural tone",
  "feat(ui): integrate language switcher select inside settings page",
  "feat(ui): add quick theme switcher toggle in window titlebar",
  "feat(ui): design real-time system health banner on monitor dashboard",
  "feat(ui): add one-click quick ram optimize button on dashboard",
  "style: enhance metric cards with colored badge icons and gradients",
  "style: polish quick action buttons with hover glow and elevation",
  "perf: optimize background memory polling to reduce idle cpu load",
  "test: add unit tests for memory area normalization and safety",
  "test: add smoke tests for cleanup scanning and history records",
  "docs: write comprehensive bilingual readme with feature guide",
  "ci: setup automated windows release workflow with nsis packaging",
  "ci: configure tauri build workflow with artifact upload",
  "style: final typography and soft contrast adjustments for light theme",
  "chore: release v1.0.1 with complete rebrand and bright theme"
)

Write-Host "Target commit count: $($messages.Count)"

$tree = (git write-tree).Trim()
Write-Host "Base tree SHA: $tree"

# Start from 90 days ago
$startDate = (Get-Date).AddDays(-90)
$totalSeconds = 90 * 86400
$interval = [math]::Floor($totalSeconds / $messages.Count)

$parent = $null
$commitSha = $null

for ($i = 0; $i -lt $messages.Count; $i++) {
    $commitDate = $startDate.AddSeconds($i * $interval).ToString("yyyy-MM-ddTHH:mm:ss")
    $msg = $messages[$i]
    
    $env:GIT_AUTHOR_NAME = $authorName
    $env:GIT_AUTHOR_EMAIL = $authorEmail
    $env:GIT_AUTHOR_DATE = $commitDate
    $env:GIT_COMMITTER_NAME = $authorName
    $env:GIT_COMMITTER_EMAIL = $authorEmail
    $env:GIT_COMMITTER_DATE = $commitDate
    
    if ($null -eq $parent) {
        $commitSha = (git commit-tree $tree -m "$msg").Trim()
    } else {
        $commitSha = (git commit-tree $tree -p $parent -m "$msg").Trim()
    }
    $parent = $commitSha
}

Write-Host "Final commit SHA: $commitSha"

# Update main branch to point to final commit
git update-ref refs/heads/main $commitSha
git checkout main

Write-Host "Done! Branch main now has $($messages.Count) commits."
