# Axis Language (Axis)

**Axis is an AI-native programming language designed for a world where code is increasingly written, transformed, and reasoned about by AI.**

This repository contains the early-stage public work on:

- **Axis Core** — a formally minimal, machine-checkable semantic kernel  
- **Axis Surface Language** — a human- and AI-friendly syntax desugared into the core  
- **Axis Bridges** — connectors into Python, Rust, JavaScript, and other host ecosystems  
- **Axis Proof Model** — foundations for machine-checked semantics and verifiable programs  
- **Axis POC Compiler** — an initial Rust implementation exploring efficiency, energy use, and parallelism

> Axis is the first language intentionally designed **with AI as a first-class user**.  
> Minimal in semantics, rich in expressivity, and engineered for correctness, safety, and massive parallelism.

---

## 🚧 Project Status

This repository is at the **very beginning** of development.  
Work in progress includes:

- Finalising the **Core semantics**  
- Defining the **Surface grammar**  
- Building a minimal **Core interpreter**  
- Creating a **Rust POC compiler** to benchmark the expected performance and energy gains  
- Designing early **bridges** (Python first, Rust next)

Expect rapid iteration, breaking changes, and evolving structure.

---

## 📂 Repository Structure (subject to change)

```

axis-lang/
├─ axis-core/          # Core semantics, VM, small-step interpreter
├─ axis-surface/       # Surface grammar + desugaring into Core
├─ axis-bridge/        # Python, Rust, JS bridges
├─ axis-compiler/      # Experimental compiler backend (Rust)
├─ docs/               # Specs, whitepapers, research notes
└─ tests/              # Non-trivial tests across all layers

```

---

## 📜 Key Ideas (very short version)

- **Immutable-by-default semantics** enabling extreme parallelism  
- **No shared mutable state** → no locks, no data races  
- **Deterministic evaluation** → reproducible computation  
- **Minimal Core** that can be formally specified and machine-checked  
- **Surface Language** rich enough for humans & AI, but fully desugars into Core  
- **Bridges** allow Axis to orchestrate real-world computation on existing stacks  
- **Proof-capable** → semantics designed to be verifiable from day one  

Full explanation coming as article series is released.

---

## 🔬 Why a Rust POC Compiler?

The Rust-based POC is designed to explore expected efficiency gains, including:

- Lock-free persistent data structures  
- Non-blocking immutability primitives  
- Task scheduling optimised around Axis semantics  
- Benchmarking CPU, memory, and energy characteristics vs. Python & Rust equivalents  

This will form the basis of the first performance article.

---

## 🤝 Contributing

This project is currently in **early research mode**, but contributions, ideas, and critique are welcome.

Please open issues for:

- Semantic questions  
- Research threads  
- Compiler explorations  
- Bridge experimentation  

A formal contribution guide will follow.

---

## 🧪 Roadmap (initial)
 
- [ ] Build reference Core interpreter  
- [ ] Define full Surface grammar  
- [ ] Implement desugaring pipeline  
- [ ] Implement Rust POC compiler  
- [ ] Build Python & Rust bridges  
- [ ] Publish articles and whitepapers  
- [ ] Begin formal semantics + proof tooling

---

## 📄 License

See [LICENSE](LICENSE.md).

---

## 📢 About

Axis is jointly developed by **Chris Taylor** and **AI collaborators**, driven by the belief that programming languages must evolve for an era where AI is a primary author of code.
