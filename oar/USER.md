# oar — User Manual

Live OAR cluster viewer. Connects to an HPC cluster over SSH and monitors jobs in real time.

---

## Startup

The app opens on the **Gantt** view and immediately begins polling the configured SSH host.

Authentication is not required to view data. Only admin operations need it:
- Create / edit / delete Gantt views
- Create / edit / delete cluster presets

**Admin credentials:** `admin` / `admin` *(proof of concept — hardcoded)*

---

## Running

### Native (desktop)

```bash
GOARD_SSH_HOST=grenoble.g5k cargo run -p oar --release
# or with a config file (sets SSH host + Gantt settings):
cargo run -p oar --release -- --config /path/to/config.toml
```

`--config <path>` loads Gantt settings and reads `[oar].ssh_host` / `[oar].oar_version` from the file. Environment variables take priority over config-file values.

### Web (WASM) — live data

**Terminal 1 - backend:**
```bash
GOARD_SSH_HOST=grenoble.g5k cargo run -p oar --release -- --serve
# or:
cargo run -p oar --release -- --config /path/to/config.toml --serve
```

**Terminal 2 - frontend:**
```bash
cd oar && trunk serve
```

Access at `http://localhost:8080`.

The frontend fetches data from the backend every 30 seconds, passing its current view window. The ⟳ button triggers an immediate fetch. If the backend is unreachable, the app falls back to mock data.

---

## Menu Bar

### File
- **Log in** / **Log out**
- **Quit**

### Options
- **Language:** English / Français
- **Font size:** 10–30
- **Save** — persists settings (native: `~/.local/share/oar/` via eframe; web: browser `localStorage`)

### Help (`?`)
Context-sensitive help for the active view.

---

## Toolbar

- **Mode:** `📊 Dashboard` / `📅 Gantt` toggle
- **Filters:** `🔎 Filters` button
- **Light/dark theme:** `☀` / `🌙`
- **Auto-refresh interval:** `30 s`, `1 min`, `5 min`, `Never`
- **Instant refresh:** `⟳` button (disabled while a refresh is in progress)

A `Refreshing data...` indicator + spinner appears at the bottom during a refresh.

---

## Job Filters

The **Filters** window filters displayed jobs by:
- **Owner**
- **Job state**
- **Cluster preset** — None or a named preset

Buttons: **Apply** / **Reset**

Filters affect: Dashboard, Gantt, XY panel.

---

## Gantt View

Interactive timeline showing live jobs and resources.

### Live Data tab

Single **Live Data** tab (always present when connected).
- `×` — stops live mode and clears data from memory

### Navigation

| Input | Action |
|-------|--------|
| Left-click drag | Horizontal pan |
| `Ctrl/Cmd + scroll` | Horizontal zoom |
| Right-click drag (vertical) | Horizontal zoom |
| `Alt/Option + scroll` | Vertical zoom |
| Left double-click | Reset view |
| Left-click on a job | Zoom to job |
| Right-click on a job | Open job details |

### Gantt toolbar
- **View** — aggregation view selector
- **⚙ Settings** — Gantt display settings panel (see [Settings](#settings))
- **Admin** — administration panel (requires auth)
- **Nav** — navigation buttons (e.g. `◀ 1d`, `1w ▶`); steps configured in Settings
- **⌚ Center on now**

---

## Settings

**⚙ Settings** opens a panel with all Gantt display options:

| Section | Controls |
|---------|----------|
| General | Truncate Absent/besteffort, min state duration, default timespan |
| Job Colors | Random / By field mode, field name, field value→color map, border |
| Gantt Rows | Default, min, max row height |
| XY Panel | Panel height, watts per resource, now-line color |
| Zoom | Max/min seconds, scroll/drag sensitivity, animation duration |
| Timeline | Show/hide year/month/day/hour/minute/second, grid auto/manual |
| Navigation | Add, remove, reorder nav step buttons |
| Layout | Gutter max width, job label field and min width, hatch spacing |
| State colors | Absent/Suspected/Dead/Standby colors (dark and light mode) |
| SSH Host | Hostname for OAR SSH connection |

Buttons at the bottom:
- **✔ Apply** — save and apply (writes to active config file)
- **Cancel** — discard changes
- **Import** — load settings from a `.toml` file into the draft (does not apply until you click Apply)
- **Export** — save the current draft to a `.toml` file

### Summary row
Shows: active view name, filtered job count, summary fields, data state (`refreshing`, `loading`, `ready`).

---

## Aggregation Views

The **View** dropdown selects the resource hierarchy. Each view defines hierarchy levels, a leaf label template, and an optional filter.

Colored bands to the left of the timeline represent hierarchy levels.

### Managing views (Admin)

Requires login as `admin`.

#### Create
**View** menu → **+ Create view**. Fill in name, levels, leaf label template, summary fields, optional filter. Click **Save view**.

#### Edit
**View** menu → hover a view → click ✏.

#### Delete
**View** menu → hover a view → click 🗑 → confirm.

### Leaf info presets

Define which fields appear when hovering a resource row. Managed from the Create/Edit view panel.

---

## XY Panel (Energy)

Secondary plot below the Gantt showing estimated power consumption from live jobs.

**Controls:**
- **Cluster filter** / **Owner filter** — filter the series
- **Reset** — clear filters
- **Fit to figure** — auto-scale Y axis
- Panning/zooming the XY plot syncs the Gantt window
- Draggable divider between Gantt and XY panel

---

## Dashboard

- Total filtered job count
- **Metrics** (colored boxes): total jobs, jobs by state, time range
- Toggle: `Show charts` / `Show metrics`
- **Job table**: column sort, pagination, column visibility
- Click a row → job detail window

---

## Cluster Presets

Cluster presets let you filter the Gantt and Dashboard to show only specific clusters.

Requires **Admin** login to manage.

From the **Admin configuration** panel:
- **New Preset** — create a preset (name + checkbox list of clusters)
- **Modify Preset** — edit an existing preset
- **Save** — saves / overwrites the preset
- **Delete** — removes the preset

Active presets appear in **Filters → Cluster preset**.

---

## Feature Summary

- Real-time OAR cluster monitoring over SSH (OAR2 and OAR3 supported via `GOARD_OAR_VERSION`)
- Auto-refresh with configurable interval (30 s / 1 min / 5 min / Never)
- Instant refresh ⟳ button
- Web (WASM) build with HTTP backend for browser access
- `--config <path>` flag for per-deployment Gantt + SSH settings
- Settings panel with Import/Export for portable config files
- Cluster filter presets (Admin)
- Interactive Gantt: zoom, pan, job detail windows
- Dashboard: metrics + chart + sortable/paginated/column-selectable table
- Aggregation views (configurable hierarchies, filters, label templates)
- XY / energy panel synchronized with Gantt
- Multi-criteria job filters
- Light/dark theme, language, font size
