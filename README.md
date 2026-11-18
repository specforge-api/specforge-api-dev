# SpecForge Dev ⚡ (Rust Engine)

> **The Universal, Zero-Code Developer Tool for API Boundary Fuzzing, Dynamic Mocking & AI Tool Testing.**

SpecForge Dev is an ultra-fast developer platform written in **Rust** and distributed as a lightweight container (`specforge/dev`). It runs alongside any backend service (Node.js/Express, Python/FastAPI, Go, Rust, Java) with **zero code modifications**, providing:

1. **Interactive Studio Web UI**: Blazing-fast API explorer, manual testing client, and real-time fuzzer dashboard on port `4000`.
2. **Schema-Aware Boundary Fuzzing**: Automatically probes edge cases (whitespace strings, negative bounds, missing keys, SQLi/XSS probes) to enforce the **"Zero 500s"** invariant.
3. **Native OpenAI Tool Schema Ingestion**: Ingests LLM function definitions (`tools: [{type: "function", ...}]`), generates OpenAPI 3.1 endpoints, and stress-tests tool executor endpoints.
4. **Dynamic Smart Mock Server**: Synthesizes schema-valid mock responses so frontend teams can develop unblocked.
5. **Polyglot Test Exporters**: Auto-generates **Jest / Vitest + Supertest** (for Node.js) and **Pytest + httpx** (for Python).

---

## 🚀 Quickstart via Docker Compose

Add the following block to your existing `docker-compose.dev.yml` (e.g. alongside Express or FastAPI):

```yaml
services:
  # Your existing backend service (FastAPI, Express, Go, etc.)
  api:
    build: .
    ports:
      - "8000:8000"
    volumes:
      - .:/app # Mount host code for instant hot-reloading

  # SpecForge Dev Engine (Rust) — Zero code modifications required
  specforge:
    image: ghcr.io/specforge-api/specforge-api-dev:latest
    container_name: specforge-dev
    restart: unless-stopped
    ports:
      - "4000:4000"   # Studio UI & Mock Server
    environment:
      - SPEC_SOURCE=http://api:8000/openapi.json
      - TARGET_URL=http://api:8000
      - EXPORT_TARGET=pytest   # or 'jest'
      - FAIL_ON_500=true
      - THEME=dark
    volumes:
      - ./tests/generated:/output
    depends_on:
      - api
```

Open **`http://localhost:4000`** in your browser to access the Studio Web UI!

---

## 🧠 In-Browser WebGPU AI & Natural Language Fuzzing

SpecForge Dev ships with zero-cloud, on-device AI running entirely inside your browser via WebGPU & WebAssembly (`SmolLM2-135M-Instruct`):

1. **✨ 1-Click "AI Randomize" (Request Body Editor):**
   - Synthesizes realistic, domain-aware payload values (emails, addresses, realistic names, bounded numeric quantities) instead of static dummy values (`"string"`, `0`).
   - Includes full JSONC (JSON with Comments) code editor with syntax line numbers and `Ctrl + /` comment shortcuts.
2. **🧪 AI Custom Test Synthesizer:**
   - Type free-form natural language test scenarios (e.g. *"Test SQL injection vulnerability in username"*, *"Test negative boundary overflow"*) or click strategy chips (`🛡️ SQLi`, `⚡ XSS`, `📉 Negative Boundary`, `🔡 Unicode/Emoji`, `🔒 Role Escalation`, `🚫 Whitespace`).
   - Auto-fills all 7 custom probe fields with glowing visual feedback.
3. **🛡️ 100% Client-Side Privacy Sandbox:**
   - Zero source code access, zero filesystem write access, zero cloud telemetry. Model weights cache in the browser's IndexedDB, keeping the container image ultra-lightweight at **24.5 MB**.

---

## 🔄 Instant Live Hot-Reloading

SpecForge watches your backend's OpenAPI contract in real time:
- Whenever backend routes or schemas change on the host disk, Docker volume mounts propagate the change to your dev server (`uvicorn --reload`, `nodemon`, etc.).
- SpecForge's background watcher polls the OpenAPI endpoint every 2 seconds, detects schema diffs, and hot-updates the Studio UI without requiring a browser page refresh.

Check out the full working sample in [sample-fastapi-project](../sample-fastapi-project/README.md).

---

## 📦 Workspace Architecture

This repository is organized as a modular Rust Cargo workspace:

```
crates/
├── specforge-core/       # Schema AST, OpenAPI 3.0/3.1 & OpenAI tool parsers
├── specforge-fuzzer/     # Combinatorial string, numeric, array, object fuzzers
├── specforge-mock/       # Dynamic mock data generator
├── specforge-exporters/  # Jest (TypeScript) & Pytest (Python) test generators
├── specforge-server/     # Axum web server, REST API & embedded Studio UI
└── specforge-cli/        # Standalone binary entrypoint (specforge serve / test)
```

---

## 🧪 Testing with OpenAI Tool Schemas

To test backend tool executors against LLM function calls:

1. Place your `ai_tools.json` inside `./specs/`:
   ```bash
   docker run -p 4000:4000 \
     -v $(pwd)/specs:/specs:ro \
     -e SPEC_SOURCE=/specs/ai_tools.json \
     -e TARGET_URL=http://host.docker.internal:8000 \
     specforge/dev:latest
   ```
2. SpecForge auto-detects the OpenAI format, synthesizes endpoints (`POST /tools/{function_name}`), and generates combinatorial boundary test cases to test how your system handles hallucinated parameters.

---

## 🛠️ Building Locally (Rust Toolchain)

```bash
# Build the entire workspace
cargo build --release

# Run unit tests
cargo test

# Launch the Dev Server
cargo run --bin specforge -- serve --spec ./examples/express-sample/index.js --port 4000
```

---

## 📄 License

SpecForge Dev is licensed under the [MIT License](LICENSE).

