#!/usr/bin/env bash
# End-to-end validation of the required workflow against a running API.
# Requires: curl, jq.   Usage: ./scripts/validate.sh [http://127.0.0.1:8080]
set -euo pipefail

API="${1:-http://127.0.0.1:8080}"

step() { printf '\n\033[36m[%s] %s\033[0m\n' "$1" "$2"; }
ok() { printf '    \033[32mOK\033[0m  %s\n' "$1"; }
fail() { printf '    \033[31mFAIL\033[0m %s\n' "$1"; exit 1; }

# api METHOD PATH [JSON_BODY] [TOKEN] -> prints body; status via $(status).
# The status goes to a file because api is usually called inside $(...) subshells.
api() {
  local method=$1 path=$2 body=${3:-} token=${4:-}
  local args=(-s -o /tmp/taskapp_body -w '%{http_code}' -X "$method" "$API$path" -H 'Content-Type: application/json')
  [[ -n $token ]] && args+=(-H "Authorization: Bearer $token")
  [[ -n $body ]] && args+=(-d "$body")
  curl "${args[@]}" > /tmp/taskapp_status
  cat /tmp/taskapp_body
}
status() { cat /tmp/taskapp_status; }

login() {
  local email=$1 password=$2 challenge code token
  challenge=$(api POST /auth/login "{\"email\":\"$email\",\"password\":\"$password\"}")
  [[ $(status) == 200 ]] || fail "login failed: $challenge"
  [[ $(jq -r '.access_token // empty' <<<"$challenge") == "" ]] || fail "JWT returned before 2FA"
  local cid; cid=$(jq -r .login_challenge_id <<<"$challenge")
  ok "login_challenge_id = $cid (no JWT returned)" >&2
  code=$(api GET "/dev/email-logs/latest?email=$email" | jq -r .code)
  ok "verification code from dev email log: $code" >&2
  token=$(api POST /auth/verify-2fa "{\"login_challenge_id\":\"$cid\",\"code\":\"$code\"}")
  [[ $(status) == 200 ]] || fail "verify failed: $token"
  ok "JWT issued for $email" >&2
  jq -r .access_token <<<"$token"
}

step 1 "Create users Admin and James Bond (POST /seed/users?reset=true)"
api POST "/seed/users?reset=true" '{}' | jq -r '.users[] | "    \(.full_name) <\(.email)> role=\(.role)"'

step 2 "Admin login + 2FA"
ADMIN=$(login admin@example.com 'Admin@123')

step 3 "Admin creates exactly 5 tasks"
IDS=()
while IFS='|' read -r title priority; do
  id=$(api POST /tasks "{\"title\":\"$title\",\"priority\":\"$priority\"}" "$ADMIN" | jq -r .id)
  [[ $(status) == 201 ]] || fail "create failed"
  IDS+=("$id"); ok "$title [$priority] $id"
done <<'EOF'
Infiltrate SPECTRE meeting|high
Collect gadgets from Q|medium
Brief M on findings|low
Audit MI6 budget|medium
Renew Aston Martin insurance|low
EOF

step 4 "Admin assigns exactly 3 tasks to James Bond"
body=$(jq -n --arg a "${IDS[0]}" --arg b "${IDS[1]}" --arg c "${IDS[2]}" \
  '{task_ids:[$a,$b,$c], assignee_email:"jamesbond@example.com"}')
api POST /tasks/assign "$body" "$ADMIN" | jq -r '"    assigned_count = \(.assigned_count) to \(.assigned_to)"'

step 5 "James Bond login + 2FA"
BOND=$(login jamesbond@example.com 'Bond@007')

step 6 "James Bond tries to create a task (expect 403)"
api POST /tasks '{"title":"Go rogue","priority":"high"}' "$BOND" >/dev/null
[[ $(status) == 403 ]] || fail "expected 403, got $(status)"
ok "403 Forbidden"

step 7 "GET /tasks/view-my-tasks (expect 3 tasks, cache.hit=false)"
first=$(api GET /tasks/view-my-tasks '' "$BOND")
[[ $(jq .summary.total_assigned_tasks <<<"$first") == 3 ]] || fail "expected 3 tasks"
[[ $(jq .cache.hit <<<"$first") == false ]] || fail "expected cache.hit=false"
ok "3 tasks, cache.hit = false"

step 8 "GET /tasks/view-my-tasks again (expect cache.hit=true)"
second=$(api GET /tasks/view-my-tasks '' "$BOND")
[[ $(jq .cache.hit <<<"$second") == true ]] || fail "expected cache.hit=true"
ok "cache.hit = true"

printf '\nFinal response:\n'
jq . <<<"$second"
printf '\n\033[32mAll validation steps passed.\033[0m\n'
