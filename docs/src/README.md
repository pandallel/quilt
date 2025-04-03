**Quilt** is a local-first, modular memory and context engine. It watches your work, fragments your documents into meaningful pieces (swatches), embeds them into a searchable memory (the swatch book), and assembles contextual spreads in response to queries.

Use Quilt to power LLM tools with fast, structured, and evolving context—without relying on cloud infrastructure or leaking your knowledge.

---

## ✨ Core Principles

- **Local-first** – Everything runs on your machine. No cloud, no leaks.
- **Modular** – Watching, swatching, embedding, and querying are decoupled and swappable.
- **Quiet** – You don’t interact with Quilt directly—it works behind the scenes to support other systems.
- **Crafted** – Inspired by the precision, care, and reuse of quilting.

---

## 🧠 Domain Concepts

| Term            | Description                                                                                 |
| --------------- | ------------------------------------------------------------------------------------------- |
| **Material**    | A raw document or file—notes, code, transcripts, etc.                                       |
| **Swatch**      | A meaningful fragment cut from a Material                                                   |
| **Swatch Book** | The searchable memory of embedded Swatches                                                  |
| **Spread**      | A contextual bundle of Swatches and their source Material, assembled in response to a Query |

---

## 🧰 What Quilt Does

- 📂 Watches one or more folders for new or updated Materials
- ✂️ Cuts Materials into Swatches based on content structure
- 🔢 Embeds Swatches using a local model (e.g. `llama.cpp`, `gguf`)
- 📚 Stores them in a fast, local Swatch Book
- 🧠 Responds to Queries by assembling contextual Spreads
