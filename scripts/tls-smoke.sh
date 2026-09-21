#!/usr/bin/env bash
set -euo pipefail

client_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
oscalify_root="${OSCALIFY_ROOT:-"$client_root/../ckodex-oscalify"}"
client_bin="${OSCAL_CLI_BIN:-}"
server_bin="${OSCALIFY_SERVER_BIN:-}"
regenerate_proto="${OSCALIFY_REGENERATE_PROTO:-0}"
port="${OSCALIFY_TLS_SMOKE_PORT:-50053}"
session="oscal-cli-tls-$$"
tmp_root="$(mktemp -d "${TMPDIR:-/tmp}/oscal-cli-tls.XXXXXX")"
server_pid=""

cleanup() {
    if [[ -n "$server_pid" ]]; then
        kill "$server_pid" 2>/dev/null || true
        wait "$server_pid" 2>/dev/null || true
    fi
    tmux kill-session -t "$session" 2>/dev/null || true
    rm -rf "$tmp_root"
}
trap cleanup EXIT

if ! command -v openssl >/dev/null 2>&1; then
    echo "openssl is required for the TLS/mTLS smoke check" >&2
    exit 1
fi
if ! command -v tmux >/dev/null 2>&1; then
    echo "tmux is required for the authenticated TUI smoke check" >&2
    exit 1
fi

if [[ ! -x "$client_bin" ]]; then
    (cd "$client_root" && cargo build --quiet)
    if resolved_bin="$("$client_root/scripts/cargo-binary-path.sh" debug)"; then
        client_bin="$resolved_bin"
    fi
fi
if [[ ! -x "$client_bin" ]]; then
    echo "client binary not found: ${client_bin:-unresolved}" >&2
    exit 1
fi
if [[ -z "$server_bin" ]]; then
    if ! command -v go >/dev/null 2>&1; then
        echo "go is required when OSCALIFY_SERVER_BIN is not supplied" >&2
        exit 1
    fi
    server_source_root="$oscalify_root"
    if [[ "$regenerate_proto" == "1" ]]; then
        if ! command -v buf >/dev/null 2>&1; then
            echo "buf is required when OSCALIFY_REGENERATE_PROTO=1" >&2
            exit 1
        fi
        server_source_root="$tmp_root/oscalify-regenerated"
        mkdir -p "$server_source_root"
        cp -R "$oscalify_root/." "$server_source_root/"
        if (cd "$server_source_root" && buf generate) >"$tmp_root/buf-generate.log" 2>&1; then
            printf '%s\n' 'disposable proto regeneration completed' >&2
        else
            printf '%s\n' 'disposable proto regeneration unavailable; using checked-in generated files in the disposable copy' >&2
            sed -n '1,120p' "$tmp_root/buf-generate.log" >&2 || true
        fi
    fi
    server_bin="$tmp_root/xoscal-server"
    (cd "$server_source_root" && go build -o "$server_bin" ./server/cmd/xoscal-server)
elif [[ ! -x "$server_bin" ]]; then
    echo "OSCALIFY_SERVER_BIN is not executable: $server_bin" >&2
    exit 1
fi

ca_key="$tmp_root/ca.key"
ca_cert="$tmp_root/ca.crt"
server_key="$tmp_root/server.key"
server_csr="$tmp_root/server.csr"
server_cert="$tmp_root/server.crt"
client_key="$tmp_root/client.key"
client_csr="$tmp_root/client.csr"
client_cert="$tmp_root/client.crt"

openssl req -x509 -newkey rsa:2048 -nodes \
    -keyout "$ca_key" -out "$ca_cert" -days 1 \
    -subj "/CN=OSCAL CLI TLS smoke CA" >/dev/null 2>&1
openssl genrsa -out "$server_key" 2048 >/dev/null 2>&1
openssl req -new -key "$server_key" -out "$server_csr" \
    -subj "/CN=localhost" >/dev/null 2>&1
cat >"$tmp_root/server.ext" <<'EOF'
basicConstraints=critical,CA:false
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=serverAuth
subjectAltName=DNS:localhost,IP:127.0.0.1
EOF
openssl x509 -req -in "$server_csr" -CA "$ca_cert" -CAkey "$ca_key" \
    -CAcreateserial -out "$server_cert" -days 1 -sha256 \
    -extfile "$tmp_root/server.ext" >/dev/null 2>&1

openssl genrsa -out "$client_key" 2048 >/dev/null 2>&1
openssl req -new -key "$client_key" -out "$client_csr" \
    -subj "/CN=oscal-cli-tls-smoke-client" >/dev/null 2>&1
cat >"$tmp_root/client.ext" <<'EOF'
basicConstraints=critical,CA:false
keyUsage=critical,digitalSignature,keyEncipherment
extendedKeyUsage=clientAuth
EOF
openssl x509 -req -in "$client_csr" -CA "$ca_cert" -CAkey "$ca_key" \
    -CAcreateserial -out "$client_cert" -days 1 -sha256 \
    -extfile "$tmp_root/client.ext" >/dev/null 2>&1
chmod 600 "$ca_key" "$server_key" "$client_key"

cat >"$tmp_root/server.yaml" <<EOF
server:
  addr: 127.0.0.1:$port
  max_recv_msg_size_mb: 4
  max_send_msg_size_mb: 4
  max_concurrent_streams: 1000
  keepalive_time: 2m
  keepalive_timeout: 20s
  enable_reflection: false
  enable_pprof: false
  pprof_addr: :6060
  shutdown_timeout: 30s
  tls_cert_path: "$server_cert"
  tls_key_path: "$server_key"
  tls_client_ca_path: "$ca_cert"
  require_tls: true
store:
  dsn: ":memory:"
  max_open_conn: 10
  max_idle_conn: 2
  conn_max_lifetime: 1h
vector:
  backend: sqlite
  uri: ""
observability:
  metrics_enabled: false
  metrics_addr: :9090
  tracing_enabled: false
  tracing_endpoint: ""
  tracing_sample_rate: 0.1
  log_level: info
  log_format: text
security:
  rate_limit_rps: 100
  rate_limit_burst: 200
  auth_mode: none
EOF

"$server_bin" -config "$tmp_root/server.yaml" >"$tmp_root/server.log" 2>&1 &
server_pid=$!

client_args=(
    --endpoint "https://127.0.0.1:$port"
    --tls-domain localhost
    --ca-cert "$ca_cert"
    --client-cert "$client_cert"
    --client-key "$client_key"
    --timeout-secs 2
)

ready=0
for _ in {1..30}; do
    if "$client_bin" "${client_args[@]}" --format json health \
        >"$tmp_root/health.json" 2>"$tmp_root/health.err"; then
        ready=1
        break
    fi
    sleep 0.2
done
if [[ "$ready" != 1 ]]; then
    echo "TLS/mTLS test server did not become ready" >&2
    sed -n '1,120p' "$tmp_root/server.log" >&2 || true
    sed -n '1,120p' "$tmp_root/health.err" >&2 || true
    exit 1
fi

rg --no-config -q '"status": "SERVING"' "$tmp_root/health.json"
"$client_bin" "${client_args[@]}" --format json proto >"$tmp_root/proto.json"
rg --no-config -q '"proto_file_count": 13' "$tmp_root/proto.json"
rg --no-config -q '"oscal_schema_version": "1.2.3"' "$tmp_root/proto.json"
"$client_bin" "${client_args[@]}" --format json model list catalog --page-size 1 \
    >"$tmp_root/model.json"
rg --no-config -q 'items|nextPageToken' "$tmp_root/model.json"

if "$client_bin" \
    --endpoint "https://127.0.0.1:$port" \
    --tls-domain localhost --ca-cert "$ca_cert" --token beta-token \
    --timeout-secs 2 --format json health >"$tmp_root/no-client-cert.json" 2>&1; then
    echo "mTLS accepted a client without a certificate" >&2
    exit 1
fi
tui_command=(env "$client_bin" "${client_args[@]}" tui)
tmux new-session -d -s "$session" "${tui_command[*]}"
sleep 1
tmux capture-pane -J -p -t "$session" >"$tmp_root/tui.txt"
rg --no-config -q 'OSCALIFY OBSERVER' "$tmp_root/tui.txt"
rg --no-config -q 'Evidence margin' "$tmp_root/tui.txt"
tmux send-keys -t "$session" q
for _ in {1..20}; do
    if ! tmux has-session -t "$session" 2>/dev/null; then
        break
    fi
    sleep 0.1
done
if tmux has-session -t "$session" 2>/dev/null; then
    echo "authenticated TUI did not exit" >&2
    exit 1
fi

if rg --no-config -q '/(Create|Update|Delete|Ingest|Upload|Sync|Publish|Propose|Resolve|Generate|Project)' \
    "$tmp_root/server.log"; then
    echo "mutation RPC observed during authenticated read-only smoke check" >&2
    exit 1
fi

printf '%s\n' 'TLS/mTLS smoke passed: CA verification, client certificate authentication, and authenticated TUI rendering succeeded.'
