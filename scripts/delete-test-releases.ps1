# Delete all v0.1.*-test releases from GitHub
# This script removes test releases that were created during development

Write-Host "🗑️ Manual cleanup required for test releases..." -ForegroundColor Yellow
Write-Host ""
Write-Host "The following releases need to be deleted manually from GitHub web interface:" -ForegroundColor Cyan
Write-Host ""

$releases = @(
    @{Id = "225341819"; Name = "v0.1.32-test" },
    @{Id = "225338756"; Name = "vx-v0.1.29-test" },
    @{Id = "225337298"; Name = "vx-v0.1.27-test" },
    @{Id = "225335132"; Name = "vx-v0.1.25-test" },
    @{Id = "225334358"; Name = "vx-v0.1.24-test" },
    @{Id = "225328341"; Name = "vx-v0.1.23-test" },
    @{Id = "225327670"; Name = "vx-v0.1.22-test" },
    @{Id = "225326978"; Name = "vx-v0.1.21-test" },
    @{Id = "225326416"; Name = "vx-v0.1.20-test" },
    @{Id = "225324748"; Name = "vx-v0.1.19-test" },
    @{Id = "225312201"; Name = "vx-v0.1.13-test" },
    @{Id = "225308135"; Name = "vx-v0.1.9-test" },
    @{Id = "225306567"; Name = "vx-v0.1.5-test" }
)

foreach ($release in $releases) {
    Write-Host "❌ $($release.Name) (ID: $($release.Id))" -ForegroundColor Red
}

Write-Host ""
Write-Host "📝 To delete these releases:" -ForegroundColor Green
Write-Host "1. Go to: https://github.com/loonghao/vx/releases" -ForegroundColor White
Write-Host "2. Find each release listed above" -ForegroundColor White
Write-Host "3. Click 'Delete' for each test release" -ForegroundColor White
Write-Host ""
Write-Host "🎯 Keep only releases with standard v0.1.0 format (no -test suffix)" -ForegroundColor Yellow
