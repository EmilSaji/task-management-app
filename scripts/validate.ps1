# End-to-end validation of the required workflow against a running API.
# Usage:  ./scripts/validate.ps1  [-ApiUrl http://127.0.0.1:8080]
param(
    [string]$ApiUrl = "http://127.0.0.1:8080"
)

$ErrorActionPreference = "Stop"

function Step($n, $text) { Write-Host "`n[$n] $text" -ForegroundColor Cyan }
function Ok($text) { Write-Host "    OK  $text" -ForegroundColor Green }
function Fail($text) { Write-Host "    FAIL $text" -ForegroundColor Red; exit 1 }

function Api($Method, $Path, $Body = $null, $Token = $null) {
    $headers = @{}
    if ($Token) { $headers["Authorization"] = "Bearer $Token" }
    $params = @{ Method = $Method; Uri = "$ApiUrl$Path"; Headers = $headers; ContentType = "application/json" }
    if ($null -ne $Body) { $params["Body"] = ($Body | ConvertTo-Json -Depth 5) }
    Invoke-RestMethod @params
}

function Login($Email, $Password) {
    $challenge = Api POST "/auth/login" @{ email = $Email; password = $Password }
    if ($challenge.PSObject.Properties.Name -contains "access_token") { Fail "login returned a JWT before 2FA" }
    Ok "login_challenge_id = $($challenge.login_challenge_id) (no JWT returned)"
    $mail = Api GET "/dev/email-logs/latest?email=$Email"
    Ok "verification code from dev email log: $($mail.code)"
    $token = Api POST "/auth/verify-2fa" @{ login_challenge_id = $challenge.login_challenge_id; code = $mail.code }
    Ok "JWT issued for $($token.user.email) (role: $($token.user.role))"
    return $token.access_token
}

Step 1 "Create users Admin and James Bond (POST /seed/users?reset=true)"
$seed = Api POST "/seed/users?reset=true" @{}
$seed.users | ForEach-Object { Ok "$($_.full_name) <$($_.email)> role=$($_.role)" }

Step 2 "Admin login + 2FA"
$admin = Login "admin@example.com" "Admin@123"

Step 3 "Admin creates exactly 5 tasks"
$specs = @(
    @{ title = "Infiltrate SPECTRE meeting"; description = "Rome, midnight"; priority = "high" },
    @{ title = "Collect gadgets from Q"; description = "Exploding pen"; priority = "medium" },
    @{ title = "Brief M on findings"; description = "Written report"; priority = "low" },
    @{ title = "Audit MI6 budget"; description = "Quarterly"; priority = "medium" },
    @{ title = "Renew Aston Martin insurance"; description = "Again"; priority = "low" }
)
$ids = @()
foreach ($spec in $specs) {
    $task = Api POST "/tasks" $spec $admin
    $ids += $task.id
    Ok "$($task.title) [$($task.priority)] $($task.id)"
}

Step 4 "Admin assigns exactly 3 tasks to James Bond"
$assign = Api POST "/tasks/assign" @{ task_ids = $ids[0..2]; assignee_email = "jamesbond@example.com" } $admin
Ok "assigned_count = $($assign.assigned_count) to $($assign.assigned_to)"

Step 5 "James Bond login + 2FA"
$bond = Login "jamesbond@example.com" "Bond@007"

Step 6 "James Bond tries to create a task (expect 403)"
try {
    Api POST "/tasks" @{ title = "Go rogue"; priority = "high" } $bond | Out-Null
    Fail "James Bond was able to create a task"
} catch {
    $status = [int]$_.Exception.Response.StatusCode
    if ($status -ne 403) { Fail "expected 403, got $status" }
    Ok "403 Forbidden"
}

Step 7 "GET /tasks/view-my-tasks as James Bond (expect 3 tasks, cache.hit=false)"
$first = Api GET "/tasks/view-my-tasks" $null $bond
if ($first.summary.total_assigned_tasks -ne 3) { Fail "expected 3 tasks" }
if ($first.cache.hit -ne $false) { Fail "expected cache.hit=false" }
Ok "3 tasks, cache.hit = false"

Step 8 "GET /tasks/view-my-tasks again (expect cache.hit=true)"
$second = Api GET "/tasks/view-my-tasks" $null $bond
if ($second.cache.hit -ne $true) { Fail "expected cache.hit=true" }
Ok "cache.hit = true"

Write-Host "`nFinal response:" -ForegroundColor Cyan
$second | ConvertTo-Json -Depth 5
Write-Host "`nAll validation steps passed." -ForegroundColor Green
