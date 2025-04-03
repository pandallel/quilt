# Sidecar

**Sidecar** is a local-first, modular memory and context server designed to work _alongside_ you. It watches your files, builds embeddings, and serves relevant context through a lightweight API—enabling LLM-based tools, assistants, or interfaces to collaborate with your own evolving knowledge.

---

## ✨ Core Principles

- **Local-first**: All processing happens on your machine—no cloud, no leaks.
- **Modular**: Chunking, embedding, search, and serving are separate and swappable.
- **Collaborative**: Sidecar supports you and your tools—it doesn’t replace or dictate.
- **Future-facing**: Built for extensibility—CLI, chat UI, plugins, browser extensions.

---

## 🧠 What Sidecar Does

- Watches one or more folders for files
- Chunks documents into meaningful pieces
- Embeds chunks using a local model (e.g. `llama.cpp`)
- Stores them in a fast vector store
- Exposes an HTTP API (MCP) for querying relevant context

---

## 🚀 MVP Goals

- [x] CLI to index folders
- [x] File watcher for auto-updating memory
- [x] Chunker that handles plain text / Markdown
- [x] Local embedding support via `llama.cpp` or `embeddings.cpp`
- [x] Lightweight MCP API (`/query`, `/status`)
- [ ] Tauri tray app + floating chat UI (optional)
