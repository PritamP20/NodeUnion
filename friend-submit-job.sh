#!/bin/bash
set -e

# NodeUnion User Job Submission - Single File Test
# Friend runs this to submit a public image to your orchestrator

echo "========================================"
echo "NodeUnion Job Submission Test"
echo "========================================"
echo ""

echo "[1/3] Preparing a public test workload..."

read -p "Enter Orchestrator URL (e.g., http://192.168.1.26:8080): " orchestrator_url
orchestrator_url="${orchestrator_url:-http://127.0.0.1:8080}"

normalize_url() {
  local raw="$1"
  raw="${raw%/}"
  if [[ "$raw" != http://* && "$raw" != https://* ]]; then
    raw="http://$raw"
  fi
  echo "$raw"
}

is_orchestrator_reachable() {
  local url="$1"
  local code
  code=$(curl -sS -o /dev/null -w "%{http_code}" "$url/health" 2>/dev/null || true)
  [[ "$code" == "200" ]]
}

orchestrator_url="$(normalize_url "$orchestrator_url")"

if ! is_orchestrator_reachable "$orchestrator_url"; then
  lan_ip=$(ipconfig getifaddr en0 2>/dev/null || true)
  if [[ -n "$lan_ip" ]]; then
    fallback_url="http://$lan_ip:8080"
    if is_orchestrator_reachable "$fallback_url"; then
      echo "Detected reachable orchestrator at $fallback_url"
      orchestrator_url="$fallback_url"
    fi
  fi
fi

if ! is_orchestrator_reachable "$orchestrator_url"; then
  echo ""
  echo "ERROR: Orchestrator is not reachable at $orchestrator_url"
  echo "Tip: run this to verify: curl -i $orchestrator_url/health"
  echo "If localhost returns HTML 404/501, use your LAN IP (example: http://10.209.76.140:8080)."
  exit 1
fi

echo "Using orchestrator: $orchestrator_url"

read -p "Enter network id (default 1): " network_id
network_id="${network_id:-1}"

read -p "Enter your wallet address (any string for testing): " wallet_address
wallet_address="${wallet_address:-test_user_$(date +%s)}"

read -p "CPU limit (0-1, default 0.1): " cpu_limit
cpu_limit="${cpu_limit:-0.1}"

read -p "RAM limit in MB (default 128): " ram_limit
ram_limit="${ram_limit:-128}"

read -p "Workload type: service or batch (default service): " workload_type
workload_type="${workload_type:-service}"

EXPOSED_PORT=""

if [ "$workload_type" = "service" ]; then
  IMAGE="busybox:1.36"
  read -p "Service port to expose (default 8080): " exposed_port
  EXPOSED_PORT="${exposed_port:-8080}"
  COMMAND="[\"/bin/sh\",\"-lc\",\"mkdir -p /www; echo NodeUnion service is running > /www/index.html; exec httpd -f -p ${EXPOSED_PORT} -h /www\"]"
else
  IMAGE="alpine:3.20"
  COMMAND='["/bin/sh","-lc","echo NodeUnion test job started; echo Timestamp: $(date); echo Hostname: $(hostname); echo Running in $HOSTNAME; echo; echo Doing a tiny compute check; sum=0; i=1; while [ \"$i\" -le 100000 ]; do sum=$((sum + i)); i=$((i + 1)); done; echo Sum result: $sum; echo; echo NodeUnion test job complete"]'
fi

echo ""
echo "[2/3] Submitting job to orchestrator..."
echo ""

if [ "$workload_type" = "service" ]; then
  JOB_PAYLOAD=$(cat <<EOF
{
  "network_id": "$network_id",
  "user_wallet": "$wallet_address",
  "image": "$IMAGE",
  "cpu_limit": $cpu_limit,
  "ram_limit_mb": $ram_limit,
  "command": $COMMAND,
  "exposed_port": $EXPOSED_PORT
}
EOF
)
else
  JOB_PAYLOAD=$(cat <<EOF
{
  "network_id": "$network_id",
  "user_wallet": "$wallet_address",
  "image": "$IMAGE",
  "cpu_limit": $cpu_limit,
  "ram_limit_mb": $ram_limit,
  "command": $COMMAND
}
EOF
)
fi

echo "Payload:"
echo "$JOB_PAYLOAD" | jq . 2>/dev/null || echo "$JOB_PAYLOAD"
echo ""

response_file=$(mktemp)
http_code=$(curl -sS -o "$response_file" -w "%{http_code}" -X POST "$orchestrator_url/jobs/submit" \
  -H "Content-Type: application/json" \
  -d "$JOB_PAYLOAD")
response=$(cat "$response_file")
rm -f "$response_file"

echo "Response:"
echo "$response" | jq . 2>/dev/null || echo "$response"

if [[ "$http_code" -ge 400 ]]; then
  echo ""
  echo "Submit failed with HTTP $http_code"
fi

echo ""
echo "[3/3] Done."
echo ""
echo "Check status with:"
echo "  curl $orchestrator_url/jobs"
echo ""
echo "Check nodes with:"
echo "  curl $orchestrator_url/nodes"
echo ""
