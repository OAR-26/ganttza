# ganttza — Developer Reference

Shared rendering library used by both `evalys` and `oar`. Pure UI and data model — no SSH, no file I/O, no energy estimation.

---

## Tech Stack

| Component | Library |
|-----------|---------|
| UI framework | egui 0.30 + eframe 0.30 + egui_extras 0.30 |
| Plots | egui_plot 0.30 |
| Serialization | serde 1.0 + serde_json 1.0 + toml 0.8 |
| Dates | chrono 0.4 + chrono-tz 0.10 |
| i18n | rust-i18n 3 |
| Random colors | rand 0.8 |
| Enum utilities | strum 0.24 + strum_macros 0.24 |
| Range sets | range-set-blaze 0.1 |
| Native file dialogs | rfd 0.15 *(native only, not WASM)* |

---

## Workspace Layout

```
Cargo.toml            — workspace root
├── ganttza/          — shared rendering library (this crate)
├── evalys/           — static file viewer binary
└── oar/              — live OAR cluster viewer binary
```

---

## Module Structure

```
ganttza/
├── config.toml                         — Gantt config (colors, timespan)
├── views.json                          — saved Gantt views + leaf info presets
└── src/
    ├── lib.rs
    ├── models/
    │   ├── data_structure/
    │   │   ├── application_context.rs  — central app state
    │   │   ├── job_data.rs             — jobs, clusters, strata, plot_series
    │   │   ├── gantt_config.rs         — config.toml loader + save/set_config_path
    │   │   ├── application_options.rs  — zoom, pan, row height
    │   │   ├── ui_preferences.rs       — font, theme, language
    │   │   ├── filters.rs              — active job filters
    │   │   ├── job.rs / resource.rs / strata.rs / marker.rs
    │   │   ├── job_sorting.rs          — sort key + comparator for job lists
    │   │   ├── view_type.rs            — ViewType enum (Gantt, Dashboard…)
    │   │   └── mod.rs
    │   └── utils/
    │       ├── date_converter.rs
    │       ├── utils.rs                — cluster/host/resource helpers
    │       ├── secret.rs               — hardcoded auth credentials
    │       └── mod.rs
    └── views/
        ├── view.rs                     — top-level render dispatch
        ├── menu/
        │   ├── menu.rs                 — menu bar (File, Options, ?)
        │   ├── tools.rs                — toolbar + Gantt summary row
        │   ├── filtering.rs            — Filters panel
        │   ├── options.rs              — Options panel (language, font, theme)
        │   ├── settings_panel.rs       — Gantt config settings panel (+ Import/Export)
        │   ├── field_colors_editor.rs
        │   └── mod.rs
        ├── main_page/
        │   ├── dashboard.rs            — Dashboard view
        │   ├── gantt/
        │   │   ├── mod.rs              — GanttChart: tabs, panels, main render
        │   │   ├── canvas.rs           — resource row + job drawing
        │   │   ├── interaction.rs      — zoom/pan (mouse + keyboard)
        │   │   ├── timeline.rs         — time axis + "now" line
        │   │   ├── labels.rs           — gutter labels
        │   │   ├── jobs.rs             — strata field resolution, resource sorting
        │   │   ├── panels.rs           — Admin, Create/Edit view, XyPanelState
        │   │   ├── xy_plot.rs          — generic XY plot (egui_plot)
        │   │   ├── energy_estimate.rs  — watt·s estimation from jobs + resources
        │   │   ├── energy_plot.rs      — energy series rendering helper
        │   │   ├── theme.rs            — colors by light/dark theme
        │   │   └── types.rs            — Options, Info, ResourceFilter, LeafInfoPreset
        │   └── mod.rs
        └── components/
            ├── gantt_job_color.rs      — JobColor, JobColorEnum (Random / ByField)
            ├── job_details.rs          — hover/click job detail popup
            ├── dashboard_components/
            │   ├── job_table.rs        — sortable job list table
            │   ├── job_table_col_selection.rs — column visibility picker
            │   ├── job_table_sorting.rs — sort state + comparators
            │   ├── metric_box.rs       — single KPI tile
            │   ├── metric_chart.rs     — small in-tile sparkline
            │   ├── metric_grid.rs      — grid layout of metric_box tiles
            │   └── mod.rs
            └── mod.rs
```

---

## State Architecture (`ApplicationContext`)

`ApplicationContext` is the central container owned by each binary and passed into every `ganttza` render call. It is split into sub-structs:

| Field | Type | Content |
|-------|------|---------|
| `data` | `JobData` | jobs, clusters, strata, markers, `plot_series` |
| `prefs` | `UiPreferences` | font, theme, language, Gantt view state |
| `filters` | `Filters` | active job filter state |
| `options` | `ApplicationOptions` | zoom, pan, row height |

Session flags (`view_type`, `user_connected`, `show_xy_panel`, `show_gantt_panel`) sit flat on `ApplicationContext`.

### `plot_series`

`JobData.plot_series: Vec<(String, Vec<(i64, f64)>)>` is the generic XY data fed to the XY panel. `ganttza` renders whatever the binary puts there:

| Binary | What it puts in `plot_series` |
|--------|-------------------------------|
| evalys | Estimated series from jobs and/or raw measured series |
| oar | Estimated series from live jobs |

---

## Gantt Views (`views.json`)

`views.json` is loaded at startup and rewritten on every Admin UI change. It can also be edited manually while the app is closed.

### Full format

```json
{
  "views": [
    {
      "name": "Nodes",
      "levels": ["site", "cluster", "host"],
      "leaf_label_template": "{host|short}",
      "sort_by_label": false,
      "summary_fields": ["cluster", "host"],
      "leaf_infos": "host_info",
      "filter": {
        "field": "production",
        "value": "YES",
        "exclude": false
      }
    }
  ],
  "leaf_info_presets": [
    {
      "id": "host_info",
      "name": "Host",
      "fields": ["network_address", "comment", "cputype", "cpuset", "nodeset"]
    }
  ]
}
```

### View fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | yes | Label shown in the View menu |
| `levels` | string[] | yes | Hierarchy levels, coarsest to finest |
| `leaf_label_template` | string \| null | no | Label template. `{field}` or `{field\|short}` (truncates before first `.`) |
| `sort_by_label` | bool | no | Sort groups by computed label instead of raw key |
| `summary_fields` | string[] | no | Fields in the summary row. Empty = last level |
| `leaf_infos` | string \| null | no | `id` of a `leaf_info_presets` entry |
| `filter` | object \| null | no | Filter on a strata field |

### Filter fields

```json
{ "field": "production", "value": "YES", "exclude": false }
```

- `exclude: false` → keep only `field == value`
- `exclude: true` → exclude when `field == value`

### Available strata fields

`site`, `cluster`, `host`, `type`, `vlan`, `disk`, `disk_id`, `nodeset`, `subnet_address`, `subnet_prefix`, `slash_16`…`slash_22`, `network_address`, `ip`, `comment`, `nodemodel`, `cputype`, `cpufreq`, `core_count`, `thread_count`, `memnode`, `gpu_model`, `chassis`, `resource_id`, `production`, `state`, `besteffort`, `deploy`, `drain`

`site` is derived automatically from the host FQDN.

---

## Configuration (`config.toml`)

Loaded by `GanttConfig::load()` at startup. All keys are **flat** (no `[gantt]` section). Unknown sections (e.g. `[oar]`, `[evalys]`) are silently ignored by `from_toml_str`, so a single unified file can serve multiple binaries.

### Full key reference

```toml
# ── General ──────────────────────────────────────────────────────────────────
standby_truncate_state_to_now   = true   # truncate open Absent intervals to now on Standby
besteffort_truncate_job_to_now  = true   # hide future portion of besteffort jobs
min_state_duration              = 2      # seconds; intervals shorter than this are skipped
default_timespan                = 21600  # initial Gantt width in seconds (21600 = 6 h)

# ── Job colors ────────────────────────────────────────────────────────────────
job_color_min   = 140        # minimum RGB component for random colors (0–255)
job_color_mode  = "random"   # "random" (hash by id) or "field" (by field value)
job_color_field = "state"    # job field used for deterministic color when mode = "field"

# ── Job bar labels ────────────────────────────────────────────────────────────
job_label_field     = "id"   # field shown inside bars; empty = no label
job_label_min_width = 30.0   # pixels; label hidden when bar narrower than this

# ── Gantt row sizing ──────────────────────────────────────────────────────────
gantt_row_height     = 20.0  # default row height (px)
gantt_row_height_min = 8.0   # Alt+scroll lower limit (px)
gantt_row_height_max = 80.0  # Alt+scroll upper limit (px)

# ── XY panel ─────────────────────────────────────────────────────────────────
xy_panel_height           = 270.0   # default panel height (px)
energy_watts_per_resource = 300.0   # estimated W per resource when no measured file loaded
now_line_color            = "#dc0000"  # color of the vertical 'now' marker

# ── Zoom limits ───────────────────────────────────────────────────────────────
zoom_max_seconds = 172800   # 2 days — zoom-out limit
zoom_min_seconds = 5.0      # zoom-in limit

# ── Interaction sensitivity ───────────────────────────────────────────────────
scroll_zoom_sensitivity  = 0.0025  # Ctrl+scroll zoom speed
drag_zoom_sensitivity    = 0.01    # right-click drag zoom speed
zoom_animation_duration  = 0.75   # seconds for "Center on now" animation

# ── Layout ────────────────────────────────────────────────────────────────────
gutter_max_width  = 650.0   # max width of resource-label column (px)
hatch_spacing     = 10.0    # pixels between diagonal lines in dead/absent overlays
job_block_border       = false  # always draw border around job blocks
job_block_border_width = 2.5    # border thickness when enabled (px)

# ── Timeline header ───────────────────────────────────────────────────────────
show_year   = true
show_month  = true
show_day    = true
show_hour   = true
show_minute = true
show_second = true

timeline_grid_auto            = true  # auto grid spacing based on zoom level
timeline_grid_manual_period_s = 3600  # fixed grid spacing when auto = false

# ── Navigation buttons ────────────────────────────────────────────────────────
# Each [[nav_steps]] entry produces one ◀/▶ button pair.
# unit: minute | hour | day | week
[[nav_steps]]
n    = 1
unit = "day"

[[nav_steps]]
n    = 1
unit = "week"

# ── State colors ──────────────────────────────────────────────────────────────
[state_colors]        # dark mode
Absent    = "#1e64dc"
Suspected = "#dc1e1e"
Dead      = "#787878"
Standby   = "#88ffff"

[state_colors_light]  # light mode
Absent    = "#1040a0"
Suspected = "#a01010"
Dead      = "#404040"
Standby   = "#008888"

# ── Per-field value→color map (used when job_color_mode = "field") ────────────
# Add one [field_colors.<fieldname>] section per field.
# Values not listed fall back to hash-based random color.
[field_colors.state]
Running   = "#00cc44"
Waiting   = "#888888"
```

### `GanttConfig` API

| Method | Description |
|--------|-------------|
| `GanttConfig::load()` | Load from `active_config_path()` (native) or embedded `config.toml` (WASM) |
| `GanttConfig::from_toml_str(s)` | Parse TOML string; ignores unknown sections |
| `config.save()` | Write to `active_config_path()` (native only) |
| `config.save_to(path)` | Write to explicit path (used by Export button) |
| `set_config_path(path)` | Override path via `OnceLock`; call once before `App::new()` |
| `config.nav_steps_s()` | Returns step durations in seconds |

### `--config` flag

Both `oar` and `evalys` accept `--config <path>`:

```
cargo run -p oar   -- --config /path/to/myconfig.toml
cargo run -p evalys -- --config /path/to/myconfig.toml
```

At startup each binary calls `ganttza::set_config_path(path)` then reads crate-specific sections (`[oar]` or `[evalys]`) from the same file. `GanttConfig::from_toml_str` ignores unknown sections, so the file can contain both without conflict.

### Extending settings from a binary

`SettingsPanel::show_with_extra(ui, app, |ui, app| { … })` renders an extra section inside the Settings window after ganttza's own sections. The closure receives the same `ui` and `ApplicationContext`. Returns `true` the frame Apply is clicked so the binary can persist its own draft at the same time.

---

## XY Panel

The XY panel is a generic secondary plot below the Gantt. `ganttza` renders whatever is in `app.data.plot_series` — it has no concept of "energy" or "estimation." The binary is responsible for populating the series.

### Gantt ↔ XY sync

When the user pans inside the XY plot, `XyPanelState::show()` returns `Option<(i64, i64)>` (new visible range). `GanttChart::render()` applies it to `options.canvas_width_s` and `options.sideways_pan_in_points`.

The separator between the Gantt and XY panel is vertically draggable; `XyPanelState.panel_height` updates on drag.

---

## Tests

Run with `cargo test -p ganttza`.

**`src/models/data_structure/job_data.rs`** — 5 tests

| Test | What it checks |
|------|----------------|
| `rebuild_populates_cluster_resource_ids` | 2 resources in same cluster → both IDs present |
| `rebuild_populates_host_resource_ids` | 2 resources on same host → both under same host key |
| `rebuild_cluster_hosts_no_duplicates` | 2 resources on same host → host listed only once |
| `rebuild_multiple_clusters` | Resources from different clusters don't mix |
| `rebuild_clears_previous_state` | Two calls with different strata → old data cleared |
