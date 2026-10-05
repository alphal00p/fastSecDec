#!/usr/bin/env bash
set -euo pipefail
if [[ $# != 2 ]]; then
  printf 'Usage: %s FASTSECDEC_CHECKOUT NEW_OUTPUT_DIRECTORY\n' "$0" >&2
  exit 2
fi
delivery_root=$(cd "$1" && pwd -P)
delivery_parent=$(cd "$(dirname "$2")" && pwd -P)
delivery_output="$delivery_parent/$(basename "$2")"
[[ -f "$delivery_root/crates/fastsecdec/Cargo.toml" ]] || { printf 'Not a FastSecDec checkout\n' >&2; exit 2; }
# Refuse reuse, including an empty directory or symlink. Existing references remain untouched.
[[ ! -e "$delivery_output" && ! -L "$delivery_output" ]] || { printf 'Output already exists: %s\n' "$delivery_output" >&2; exit 2; }
for command in git sha256sum; do command -v "$command" >/dev/null; done
mkdir "$delivery_output"
printf 'Preparing; a failed attempt is retained for inspection.\n' > "$delivery_output/status.txt"
patch_root="$delivery_root/docs/dependency-patches"
cat > "$delivery_output/patches.sha256" <<'HASHES'
a5340eaa6d6adb1d09ca32126bd4a46ccf59fbb294f981d07acbc9e6a166b880  feynkit-literal-kinematic-substitution.patch
dd78dedecf82d3be63bbab0afcb1c5d518c1f39aa5524a14d1c70353962a4537  oneloop-symjit-2.26.4.patch
3bbda58479ffaf481c19b154b11cbf21eeff20753950a4e29a9bfe011610bbce  symbolica-attributed-alias-reference.patch
303c268012d77481a1cc6c715885faa694b987a45f6b6fbe155075f783f1380f  symbolica-canonical-complex-product.patch
c676d4329905d1c7326f312d60fe6ca5667ddc635692ba1ab281926b311772ff  symbolica-evaluator-ir-validation.patch
d73065b18e1bd4549cb8f9fc5f017e2ff5fc2c9110e41888375177a1ffb8f14c  symbolica-fallible-coefficient-map.patch
cc60a115e6ad6a30ac0bc9cc78e881a7b0da6097b1447b63372f916365a5bb0a  symbolica-fixed-argument-constant-domain.patch
5db545851888483a42004eca7550e5f86126481660537f054285378f231f8ceb  symbolica-fixed-variable-coefficient-field.patch
112d93c9c900ef0929045c909cdc78b177a64c7d20fca7356dfbf13b64cdc8bb  symbolica-literal-series-variable.patch
HASHES
(cd "$patch_root" && sha256sum --check "$delivery_output/patches.sha256")
fetch_exact() {
  local name=$1 url=$2 revision=$3 destination="$delivery_output/$1"
  git init --quiet "$destination"
  git -C "$destination" remote add origin "$url"
  git -C "$destination" fetch --quiet --depth=1 origin "$revision"
  git -C "$destination" checkout --quiet --detach FETCH_HEAD
  [[ $(git -C "$destination" rev-parse HEAD) == "$revision" ]]
  printf '%s\t%s\t%s\n' "$name" "$url" "$revision" >> "$delivery_output/repositories.tsv"
}
fetch_exact feynkit https://github.com/alphal00p/gammaloop 6c707c6b77a437256eb1180da13d4d327b371d13
fetch_exact symbolica https://github.com/symbolica-dev/symbolica 98794d0d7337ba2b08e4c046dde584ad7fc1ce10
fetch_exact numerica https://github.com/ValentinHirschi/numerica bb996e415bee9ae2c143f97408675d3051c3c2aa
fetch_exact oneloop https://github.com/alphal00p/oneloopmaster a42a60aa5fe0b3ba0a5b9bb37a17c8465c06ba5a
fetch_exact one-loop-reduce https://github.com/lcnbr/one-loop-reduce b53a70776a43bd14c6562c52a03bc4909568e473
apply_patch() {
  local owner=$1 patch=$2
  git -C "$delivery_output/$owner" apply --check "$patch_root/$patch"
  git -C "$delivery_output/$owner" apply "$patch_root/$patch"
}
apply_patch feynkit feynkit-literal-kinematic-substitution.patch
for patch in \
  symbolica-fixed-argument-constant-domain.patch \
  symbolica-canonical-complex-product.patch \
  symbolica-attributed-alias-reference.patch \
  symbolica-literal-series-variable.patch \
  symbolica-evaluator-ir-validation.patch \
  symbolica-fallible-coefficient-map.patch \
  symbolica-fixed-variable-coefficient-field.patch; do
  apply_patch symbolica "$patch"
done
apply_patch oneloop oneloop-symjit-2.26.4.patch
# Absolute paths belong only to this generated config. Pass it explicitly to Cargo/maturin.
toml_string() {
  local value=$1
  [[ "$value" != *$'\n'* && "$value" != *$'\r'* && "$value" != *$'\t'* ]] || { printf 'Control character in path\n' >&2; exit 2; }
  value=${value//\\/\\\\}; value=${value//\"/\\\"}
  printf '"%s"' "$value"
}
path_patch() { printf '%s = { path = %s }\n' "$1" "$(toml_string "$2")"; }
write_overlay() {
  local scope=$1
  local packages
  printf '[patch.crates-io]\n'
  path_patch symbolica "$delivery_output/symbolica"
  path_patch graphica "$delivery_output/symbolica/lib/graphica"
  path_patch numerica "$delivery_output/numerica"
  printf '\n[patch."https://github.com/alphal00p/gammaloop"]\n'
  case "$scope" in
    root) packages=(feynkit-amplitude feynkit-generator feynkit-graph feynkit-kinematics feynkit-model feynkit-tensor idenso linnet spenso spenso-macros symbolica-utils) ;;
    portable) packages=(feynkit-graph feynkit-kinematics feynkit-model idenso linnet spenso spenso-macros symbolica-utils) ;;
    community) packages=(feynkit-amplitude feynkit-cff feynkit-generator feynkit-graph feynkit-kinematics feynkit-model feynkit-py feynkit-tensor feynkit-ufo idenso linnet linnest spenso spenso-macros spynso3 symbolica-utils typst-renderer) ;;
    *) printf 'Unknown consumer scope: %s\n' "$scope" >&2; return 2 ;;
  esac
  for package in "${packages[@]}"; do
    path_patch "$package" "$delivery_output/feynkit/crates/$package"
  done
  if [[ "$scope" != portable ]]; then
  printf '\n[patch."https://github.com/alphal00p/oneloopmaster"]\n'
  path_patch oneloop "$delivery_output/oneloop"
  [[ "$scope" != community ]] || path_patch oneloop-python "$delivery_output/oneloop/python"
  printf '\n[patch."https://github.com/lcnbr/one-loop-reduce"]\n'
  path_patch one-loop-reduce "$delivery_output/one-loop-reduce/crates/one-loop-reduce"
  [[ "$scope" != community ]] || path_patch one-loop-reduce-python "$delivery_output/one-loop-reduce/crates/one-loop-reduce-python"
  fi
  if [[ "$scope" == community ]]; then
    printf '\n[patch."https://github.com/alphal00p/fastSecDec"]\n'
    path_patch fastsecdec "$delivery_root/crates/fastsecdec"
    path_patch fastsecdec-sectors "$delivery_root/crates/fastsecdec-sectors"
  fi
  printf '\n[env]\n'
  for pair in FEYNKIT:feynkit SYMBOLICA:symbolica NUMERICA:numerica; do
    printf 'FASTSECDEC_%s_SOURCE_ROOT = { value = %s, force = true }\n' "${pair%%:*}" "$(toml_string "$delivery_output/${pair#*:}")"
  done
}
# Unused patches from several source groups make Cargo 1.98 rewrite their lockfile
# order across invocations. Keep optional reference/Python roots consumer-specific.
write_overlay community > "$delivery_output/overlay.toml"
write_overlay root > "$delivery_output/overlay-root.toml"
write_overlay portable > "$delivery_output/overlay-portable.toml"
for owner in feynkit symbolica numerica oneloop one-loop-reduce; do
  git -C "$delivery_output/$owner" diff --binary > "$delivery_output/$owner.patch"
  git -C "$delivery_output/$owner" status --porcelain > "$delivery_output/$owner.status"
done
printf 'Prepared pinned source owners and local reviewed patches. Build/runtime gates remain separate.\n' > "$delivery_output/status.txt"
printf 'Prepared overlay: %s\n' "$delivery_output/overlay.toml"
printf 'Root CLI/tests: cargo --config "%s" metadata --format-version 1\n' "$delivery_output/overlay-root.toml"
printf 'Community: use overlay.toml; portable-kernel consumer: use overlay-portable.toml.\n'
printf "First resolve deliberately and inspect the lockfile/identity diff; subsequent build gates must use --locked.\n"
