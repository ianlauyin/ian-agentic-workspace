#!/usr/bin/env bash
# Read, answer and ack Orca orchestration messages without losing a question.
#
#   orca-inbox.sh read  <run> [timeout_ms]   wait for messages, save the batch, print a summary
#   orca-inbox.sh reply <run> <msg_id> <body|->   answer one question of the saved batch
#   orca-inbox.sh ack   <run>                ack the saved batch after every question has a reply
#
# timeout_ms 0 reads without waiting. A body of "-" is read from stdin.
set -euo pipefail

ORCA="${ORCA:-orca}"
STATE_ROOT="${ORCA_INBOX_DIR:-${TMPDIR:-/tmp}/orca-inbox}"
WAIT_TYPES="worker_done,escalation,question"

die() {
  echo "orca-inbox: $*" >&2
  exit 1
}

usage() {
  sed -n '2,8p' "$0" | sed 's/^# \{0,1\}//' >&2
  exit 2
}

# Run orca and print its result document. Orca prints keepalive lines before the result when it
# waits, so keep only the last document that has "ok". Show the raw output when there is none.
orca_result() {
  local raw json
  raw="$("$ORCA" "$@" 2>&1)" || true
  json="$(jq -s 'map(select(type == "object" and has("ok"))) | last // empty' <<<"$raw" 2>/dev/null)" || json=""
  if [ -z "$json" ]; then
    echo "orca-inbox: no JSON result from: $ORCA $*" >&2
    printf '%s\n' "$raw" >&2
    exit 1
  fi
  printf '%s\n' "$json"
}

# Print the error of a failed Orca result and stop.
require_ok() {
  local json="$1" what="$2"
  if [ "$(jq -r '.ok' <<<"$json")" != "true" ]; then
    echo "orca-inbox: $what failed:" >&2
    jq '.error // .' <<<"$json" >&2
    if [ "$(jq -r '.error.code // empty' <<<"$json")" = "consumer_fenced" ]; then
      echo "orca-inbox: hint: this terminal is bound to a different run. Use: $ORCA orchestration run-use --id <run>" >&2
    fi
    exit 1
  fi
}

state_dir() {
  local dir="$STATE_ROOT/$1"
  mkdir -p "$dir"
  echo "$dir"
}

cmd_read() {
  local run="${1:-}" timeout="${2:-580000}"
  [ -n "$run" ] || usage
  local dir json args
  dir="$(state_dir "$run")"
  args=(orchestration check --run "$run" --json)
  if [ "$timeout" != "0" ]; then
    args+=(--wait --types "$WAIT_TYPES" --timeout-ms "$timeout")
  fi
  json="$(orca_result "${args[@]}")"
  require_ok "$json" "check"

  local delivery old_delivery=""
  delivery="$(jq -r '.result.deliveryId // empty' <<<"$json")"
  [ -f "$dir/batch.json" ] && old_delivery="$(jq -r '.result.deliveryId // empty' "$dir/batch.json")"
  # A replayed batch keeps its recorded replies. A new batch starts with none.
  if [ "$delivery" != "$old_delivery" ]; then
    : >"$dir/answered"
  fi
  printf '%s\n' "$json" >"$dir/batch.json"

  if [ -z "$delivery" ]; then
    echo "no messages (timedOut=$(jq -r '.result.timedOut // false' <<<"$json"))"
    return 0
  fi
  echo "delivery $delivery ($(jq -r '.result.count' <<<"$json") messages, replayed=$(jq -r '.result.replayed' <<<"$json"))"
  jq -r '.result.messages[] | select(.type != "heartbeat")
    | "\n[\(.type)] \(.id) from \(.from_handle)\n  subject: \(.subject)\n  body: \(.body)"' <<<"$json"
  local open
  open="$(open_questions "$dir" | paste -sd " " -)"
  if [ -n "$open" ]; then
    echo
    echo "open questions: $open"
  fi
}

open_questions() {
  local dir="$1"
  jq -r '.result.messages[]? | select(.type == "question") | .id' "$dir/batch.json" |
    grep -vxF -f "$dir/answered" || true
}

cmd_reply() {
  local run="${1:-}" id="${2:-}" body="${3:-}"
  [ -n "$run" ] && [ -n "$id" ] && [ -n "$body" ] || usage
  if [ "$body" = "-" ]; then
    body="$(cat)"
    [ -n "$body" ] || die "reply: empty body on stdin"
  fi
  local dir
  dir="$STATE_ROOT/$run"
  [ -f "$dir/batch.json" ] || die "reply: no saved batch for $run. Run: $0 read $run"
  local type
  type="$(jq -r --arg id "$id" '.result.messages[]? | select(.id == $id) | .type' "$dir/batch.json")"
  [ -n "$type" ] || die "reply: $id is not in the saved batch of $run"
  [ "$type" = "question" ] || die "reply: $id is a $type, not a question"

  local json
  json="$(orca_result orchestration reply --run "$run" --id "$id" --body "$body" --json)"
  require_ok "$json" "reply to $id"
  echo "$id" >>"$dir/answered"
  echo "replied to $id"
}

cmd_ack() {
  local run="${1:-}"
  [ -n "$run" ] || usage
  local dir
  dir="$STATE_ROOT/$run"
  [ -f "$dir/batch.json" ] || die "ack: no saved batch for $run. Run: $0 read $run"
  local delivery
  delivery="$(jq -r '.result.deliveryId // empty' "$dir/batch.json")"
  [ -n "$delivery" ] || die "ack: the saved batch has no delivery"
  local open
  open="$(open_questions "$dir" | paste -sd " " -)"
  [ -z "$open" ] || die "ack: these questions have no reply: $open"

  local json
  json="$(orca_result orchestration check --run "$run" --ack "$delivery" --json)"
  require_ok "$json" "ack of $delivery"
  rm -f "$dir/batch.json" "$dir/answered"
  echo "acked $delivery"
}

case "${1:-}" in
  read) shift; cmd_read "$@" ;;
  reply) shift; cmd_reply "$@" ;;
  ack) shift; cmd_ack "$@" ;;
  *) usage ;;
esac
