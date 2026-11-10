root := justfile_directory()
python := if os_family() == "windows" { "python" } else { "python3" }
renders := "tools/decode-check/target/renders"

export TYPST_ROOT := root

[private]
default:
  @just --list --unsorted

# generate the README example image
doc:
  typst compile docs/example.typ example.svg

# run test suite
test *args:
  tt run --no-fail-fast {{ args }}

# update test cases
update *args:
  tt update {{ args }}

# check formatting and lints
lint:
  typstyle --check src docs tests tools
  cargo fmt --check --manifest-path tools/decode-check/Cargo.toml
  cargo clippy --quiet --manifest-path tools/decode-check/Cargo.toml -- -D warnings
  ruff check tools
  ruff format --check tools

# apply formatting and lint fixes
fix:
  typstyle -i src docs tests tools
  cargo fmt --manifest-path tools/decode-check/Cargo.toml
  ruff check --fix tools
  ruff format tools

# regenerate the safe graphic sizes from ISO/IEC 18004
sizes:
  {{ python }} tools/safe-sizes.py --output src/safe-sizes.toml

# check the safe graphic sizes with real decoders
check-sizes *args:
  cargo run --release --quiet --manifest-path tools/decode-check/Cargo.toml -- {{ args }}

# check that codes rendered by Typst decode
check-renders:
  rm -rf {{ renders }}
  mkdir -p {{ renders }}
  typst compile --ppi 72 tools/decode-check/renders.typ "{{ renders }}/{0p}.png"
  typst eval 'query(<payload>).map(it => it.value)' --in tools/decode-check/renders.typ > {{ renders }}/payloads.json
  cargo run --release --quiet --manifest-path tools/decode-check/Cargo.toml -- --renders {{ renders }}

# package the library into the specified destination folder
package target:
  ./scripts/package "{{target}}"

# install the library with the "@local" prefix
install: (package "@local")

# install the library with the "@preview" prefix (for pre-release testing)
install-preview: (package "@preview")

[private]
remove target:
  ./scripts/uninstall "{{target}}"

# uninstalls the library from the "@local" prefix
uninstall: (remove "@local")

# uninstalls the library from the "@preview" prefix (for pre-release testing)
uninstall-preview: (remove "@preview")

# run ci suite
ci: lint test doc check-renders
