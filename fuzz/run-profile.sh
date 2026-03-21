#!/usr/bin/env bash
set -euo pipefail

profile="${1:-quick}"

targets=(
  type_unify_ops
  type_scheme_ops
  type_resolve_only
  lexer_only
  parser_roundtrip
  full_source_pipeline
  full_source_with_imports
  ir_pipeline
  linker_inputs
  custom_section_decoders
  tooling_positions
)

case "$profile" in
  quick)
    fuzz_args=("-runs=256")
    ;;
  smoke)
    fuzz_args=("-max_total_time=60")
    ;;
  long)
    fuzz_args=("-max_total_time=600")
    ;;
  *)
    echo "unknown profile: $profile" >&2
    echo "usage: $0 [quick|smoke|long]" >&2
    exit 1
    ;;
esac

dict_targets=(
  type_resolve_only
  lexer_only
  parser_roundtrip
  full_source_pipeline
  full_source_with_imports
  ir_pipeline
)

has_dict_target() {
  local target="$1"
  for candidate in "${dict_targets[@]}"; do
    if [[ "$candidate" == "$target" ]]; then
      return 0
    fi
  done
  return 1
}

for target in "${targets[@]}"; do
  echo "==> $target ($profile)"
  cmd=(cargo fuzz run "$target" "fuzz/corpus/$target" --)
  if has_dict_target "$target"; then
    cmd+=("-dict=fuzz/dictionaries/halcyon.dict")
  fi
  cmd+=("${fuzz_args[@]}")
  "${cmd[@]}"
done
