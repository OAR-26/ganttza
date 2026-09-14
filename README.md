# Goard

Gantt-based dashboard for monitoring HPC cluster jobs, built with Rust + egui/eframe.

| Crate | Purpose |
|-------|---------|
| **evalys** | Import and visualize static OAR/energy JSON files |
| **oar** | Connect to a live OAR cluster over SSH in real time |
| **ganttza** | Shared rendering library (Gantt, Dashboard, XY panel, filters) |

---

## Documentation

| | User manual | Developer reference |
|-|-------------|---------------------|
| **evalys** | [evalys/USER.md](evalys/USER.md) | [evalys/DEV.md](evalys/DEV.md) |
| **oar** | [oar/USER.md](oar/USER.md) | [oar/DEV.md](oar/DEV.md) |
| **ganttza** | — | [ganttza/DEV.md](ganttza/DEV.md) |

---

## Quick Start

### Prerequisites

- Rust + Cargo
- SSH access to an HPC cluster *(oar only)*

### evalys (static file viewer)

```bash
cargo run -p evalys --release
cargo run -p evalys --release -- examples/oar.json
cargo run -p evalys --release -- examples/oar.json+examples/energy.json
```

### oar - native

```bash
GOARD_SSH_HOST=grenoble.g5k cargo run -p oar --release
```

### oar - web (WASM)

```bash
# Terminal 1 - backend (SSH + HTTP server)
GOARD_SSH_HOST=grenoble.g5k cargo run -p oar --release -- --serve

# Terminal 2 - frontend (WASM in browser)
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
cd oar && trunk serve
```

Open `http://localhost:8080`. Replace `localhost` with the machine IP for other devices on the same network.

See [oar/DEV.md](oar/DEV.md) for the full web architecture.

---

## License

LGPL-2.1
