#!/bin/bash

set -u

wasi="./target/wasm32-wasip1/release-wasi/pmtxt2merged.wasm"

hostdir="./sample.d"
guestdir='/guest.d'

smet='/guest.d/state.txt'
nmet='/guest.d/next.txt'

genhelp(){
  name=$1
  msg=$2

  printf '# HELP %s %s\n' "${name}" "${msg}"
}

gentype(){
  name=$1
  typ=$2

  printf '# TYPE %s %s\n' "${name}" "${typ}"
}

gen1(){
  dev=$1
  val=$2

  printf 'node_network_mtu_bytes{device="%s"} %s\n' \
    "${dev}" \
    "${val}"
}

gen2(){
  cpu=$1
  mode=$2
  val=$3

  printf 'node_cpu_seconds_total{cpu="%s",mode="%s"} %s\n' \
    "${cpu}" \
    "${mode}" \
    "${val}"
}

geninput(){
  echo generating input...

  mkdir -p "${hostdir}"

  genhelp node_network_mtu_bytes 'mtu_bytes value of /sys/class/net/<iface>.' \
    > "${hostdir}/state.txt"
  gentype node_network_mtu_bytes 'gauge' >> "${hostdir}/state.txt"
  gen1 lo     65536 >> "${hostdir}/state.txt"
  gen1 virbr0 1500 >> "${hostdir}/state.txt"

  genhelp node_cpu_seconds_total 'Seconds the CPUs spent in each mode.' \
    >> "${hostdir}/state.txt"
  gentype node_cpu_seconds_total 'counter' >> "${hostdir}/state.txt"
  gen2 0 iowait  28462.45 >> "${hostdir}/state.txt"
  gen2 0 system 107437.36 >> "${hostdir}/state.txt"
  gen2 0 user   188022.48 >> "${hostdir}/state.txt"
  gen2 1 iowait  24010.53 >> "${hostdir}/state.txt"
  gen2 1 system  20281.83 >> "${hostdir}/state.txt"
  gen2 1 user    27676.82 >> "${hostdir}/state.txt"

  genhelp node_network_mtu_bytes 'mtu_bytes value of /sys/class/net/<iface>.' \
    > "${hostdir}/next.txt"
  gentype node_network_mtu_bytes 'gauge' >> "${hostdir}/next.txt"
  gen1 lo     65536 >> "${hostdir}/next.txt"
  gen1 virbr0 1500 >> "${hostdir}/next.txt"

  genhelp node_cpu_seconds_total 'Seconds the CPUs spent in each mode.' \
    >> "${hostdir}/next.txt"
  gentype node_cpu_seconds_total 'counter' >> "${hostdir}/next.txt"
  gen2 0 iowait  0.0001 >> "${hostdir}/next.txt"
  gen2 0 system  0.0002 >> "${hostdir}/next.txt"
  gen2 0 user    0.0003 >> "${hostdir}/next.txt"
  gen2 1 iowait  0.0004 >> "${hostdir}/next.txt"
  gen2 1 system  0.0005 >> "${hostdir}/next.txt"
  gen2 1 user    0.0006 >> "${hostdir}/next.txt"
}

run_wasi(){
  wasmtime \
    run \
    --env ENV_STATE_MET_TXT_FILENAME="${smet}" \
    --env ENV_NEXT_MET_TXT_FILENAME="${nmet}" \
    --dir "${hostdir}::${guestdir}" \
    "${wasi}"
}

test -f "${hostdir}/state.txt" || geninput
test -f "${hostdir}/next.txt" || geninput

sort -nk 2,2 "${hostdir}/state.txt" | bat --language=toml
echo
echo

run_wasi | sort -nk 2,2 | bat --language=toml
