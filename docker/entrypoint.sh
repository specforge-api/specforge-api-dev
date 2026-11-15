#!/bin/sh
set -e

echo "⚡ Starting SpecForge Dev (Rust Engine)..."

# Auto-detect specification source if not explicitly provided
if [ -z "$SPEC_SOURCE" ]; then
  if [ -f "/specs/openapi.json" ]; then
    SPEC_SOURCE="/specs/openapi.json"
  elif [ -f "/specs/openapi.yaml" ]; then
    SPEC_SOURCE="/specs/openapi.yaml"
  elif [ -f "/specs/swagger.json" ]; then
    SPEC_SOURCE="/specs/swagger.json"
  elif [ -f "/specs/ai_tools.json" ]; then
    SPEC_SOURCE="/specs/ai_tools.json"
  else
    SPEC_SOURCE="/specs/openapi.json"
  fi
fi

if [ "$1" = "serve" ]; then
  exec specforge serve \
    --spec "$SPEC_SOURCE" \
    --host "${SPECFORGE_HOST:-0.0.0.0}" \
    --port "${SPECFORGE_PORT:-4000}" \
    --target-url "${TARGET_URL:-http://localhost:8000}" \
    --export-target "${EXPORT_TARGET:-pytest}" \
    --theme "${THEME:-dark}"
fi

exec specforge "$@"

