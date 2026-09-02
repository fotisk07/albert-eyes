#!/bin/sh
set -eu

[ "$(id -u)" -eq 0 ] || {
    echo "collect-smart.sh must run as root" >&2
    exit 1
}

AL_DEVICE=/dev/disk/by-id/ata-WDC_WD10EZRX-00A8LB0_WD-WMC1U5466748
BERT_DEVICE=/dev/disk/by-id/ata-KXG50ZNV512G_NVMe_TOSHIBA_512GB_681B703XKAWP
CACHE_DIR=/run/albert-eyes
CACHE_FILE=$CACHE_DIR/smart.json

al=$(/usr/sbin/smartctl -a -d sat "$AL_DEVICE" 2>/dev/null || true)
bert=$(/usr/sbin/smartctl -a "$BERT_DEVICE" 2>/dev/null || true)

al_temp=$(printf '%s\n' "$al" | awk '/Temperature_Celsius/ { print $10; exit }')
bert_temp=$(printf '%s\n' "$bert" | awk '/^Temperature:/ { print $2; exit }')

al_health=$(printf '%s\n' "$al" | awk -F: '/SMART overall-health/ { gsub(/ /, "", $2); print $2; exit }')
bert_health=$(printf '%s\n' "$bert" | awk -F: '/SMART overall-health/ { gsub(/ /, "", $2); print $2; exit }')

[ -n "$al_temp" ] || al_temp=null
[ -n "$bert_temp" ] || bert_temp=null

as_json_bool() {
    case "$1" in
        PASSED) echo true ;;
        FAILED*) echo false ;;
        *) echo null ;;
    esac
}

al_health=$(as_json_bool "$al_health")
bert_health=$(as_json_bool "$bert_health")

install -d -o root -g root -m 0755 "$CACHE_DIR"
temporary=$(mktemp "$CACHE_DIR/.smart.json.XXXXXX")
trap 'rm -f "$temporary"' EXIT HUP INT TERM

printf '{"al":{"temperature_c":%s,"healthy":%s},"bert":{"temperature_c":%s,"healthy":%s}}\n' \
    "$al_temp" "$al_health" "$bert_temp" "$bert_health" >"$temporary"

chmod 0644 "$temporary"
mv -f "$temporary" "$CACHE_FILE"
trap - EXIT
