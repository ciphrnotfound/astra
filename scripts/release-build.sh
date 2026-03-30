#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

version="${1:-}"
if [[ -z "${version}" ]]; then
  version="$(node -e "process.stdout.write(require('./package.json').version)")"
fi

platform="$(uname -s)"
arch="$(uname -m)"

platKey="linux"
if [[ "${platform}" == "Darwin" ]]; then platKey="darwin"; fi
if [[ "${platform}" == "MINGW"* || "${platform}" == "MSYS"* || "${platform}" == "CYGWIN"* ]]; then platKey="win32"; fi

archKey="x64"
if [[ "${arch}" == "aarch64" || "${arch}" == "arm64" ]]; then archKey="arm64"; fi

cargo build -p astra-cli --release

ext="bin"
src="${root}/target/release/astra-cli"
if [[ "${platKey}" == "win32" ]]; then
  ext="exe"
  src="${root}/target/release/astra-cli.exe"
fi

name="astra-cli-${version}-${platKey}-${archKey}.${ext}"
mkdir -p "${root}/dist"
cp "${src}" "${root}/dist/${name}"
chmod +x "${root}/dist/${name}" || true
echo "${name}"
