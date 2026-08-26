use super::theme::get_theme_colors;
use super::types::{Info, Options};
use chrono::{DateTime, Local};
use egui::{pos2, remap_clamp, Align2, Color32, Rgba, Rect, Stroke};

#[derive(Clone, Copy)]
struct GridLevel {
    step_s: i64,
    medium_s: i64,
    big_s: i64,
}

// Sorted smallest→largest step_s.
// Labels rendered at medium_s (medium alpha) and big_s (big alpha).
// pick_grid_level selects the finest level where canvas_width_s / step_s <= max_lines.
const GRID_LEVELS: &[GridLevel] = &[
    GridLevel { step_s: 1,      medium_s: 1,       big_s: 5       }, // 1s / 5s
    GridLevel { step_s: 1,      medium_s: 5,       big_s: 30      }, // 5s / 30s
    GridLevel { step_s: 2,      medium_s: 10,      big_s: 60      }, // 10s / 1min
    GridLevel { step_s: 3,      medium_s: 30,      big_s: 60      }, // 30s / 1min
    GridLevel { step_s: 6,      medium_s: 60,      big_s: 300     }, // 1min / 5min
    GridLevel { step_s: 30,     medium_s: 300,     big_s: 600     }, // 5min / 10min
    GridLevel { step_s: 90,     medium_s: 900,     big_s: 1800    }, // 15min / 30min
    GridLevel { step_s: 180,    medium_s: 1800,    big_s: 3600    }, // 30min / 1h
    GridLevel { step_s: 360,    medium_s: 3600,    big_s: 7200    }, // 1h / 2h
    GridLevel { step_s: 720,    medium_s: 7200,    big_s: 14400   }, // 2h / 4h
    GridLevel { step_s: 1800,   medium_s: 18000,   big_s: 36000   }, // 5h / 10h
    GridLevel { step_s: 7200,   medium_s: 72000,   big_s: 144000  }, // 20h / 40h
    GridLevel { step_s: 86400,    medium_s: 604800,   big_s: 1209600  }, // 1wk / 2wk
    GridLevel { step_s: 604800,   medium_s: 2592000,  big_s: 5184000  }, // 1mo / 2mo
    GridLevel { step_s: 2592000,  medium_s: 5184000,  big_s: 10368000 }, // 2mo / 4mo
    GridLevel { step_s: 5184000,  medium_s: 10368000, big_s: 20736000 }, // 4mo / 8mo
    GridLevel { step_s: 10368000, medium_s: 31536000, big_s: 63072000 }, // 1y / 2y
];

pub(super) fn timeline_header_height(options: &Options, usable_width: f32, text_height: f32) -> f32 {
    let max_lines = usable_width / 13.0;
    let level = pick_grid_level(options, max_lines);
    let split = level.medium_s < 3600;
    text_height * rows_shown(options, split) as f32 + 5.0
}

fn rows_shown(options: &Options, split: bool) -> u8 {
    let date_shown = options.show_year || options.show_month || options.show_day;
    let time_shown = options.show_hour || options.show_minute || options.show_second;
    if split {
        date_shown as u8 + time_shown as u8
    } else if date_shown || time_shown {
        1
    } else {
        0
    }
}

fn pick_grid_level(options: &Options, max_lines: f32) -> GridLevel {
    if !options.timeline_grid_auto {
        // Manual mode: one fixed period, every line drawn at full (big) alpha.
        let period_s = options.timeline_grid_manual_period_s.max(1);
        return GridLevel { step_s: period_s, medium_s: period_s, big_s: period_s };
    }
    // Target ~max_lines/10 medium labels on screen → ~130px spacing for any level.
    let max_labels = max_lines / 10.0;
    for &lvl in GRID_LEVELS {
        if options.canvas_width_s / (lvl.medium_s as f32) <= max_labels {
            return lvl;
        }
    }
    *GRID_LEVELS.last().unwrap()
}

pub(super) fn paint_timeline_text(
    info: &Info,
    canvas: Rect,
    options: &Options,
    level: GridLevel,
    fixed_timeline_y: f32,
    alpha_multiplier: f32,
    zoom_factor: f32,
) -> Vec<egui::Shape> {
    let mut shapes = vec![];
    let big_alpha = remap_clamp(zoom_factor, 0.0..=1.0, 0.5..=1.0);
    let medium_alpha = remap_clamp(zoom_factor, 0.0..=1.0, 0.1..=0.5);

    // Start at the first grid line at or before the visible left edge.
    let visible_left_s = info.start_s
        - ((options.sideways_pan_in_points / info.usable_width()) * options.canvas_width_s) as i64;
    let mut grid_s = (visible_left_s / level.step_s) * level.step_s;

    loop {
        let line_x = info.point_from_s(options, grid_s);

        if line_x > canvas.max.x {
            break;
        }

        if canvas.min.x <= line_x {
            let big_line   = grid_s % level.big_s   == 0;
            let medium_line = grid_s % level.medium_s == 0;

            let text_alpha = if big_line {
                big_alpha
            } else if medium_line {
                medium_alpha
            } else {
                0.0
            };

            if text_alpha > 0.0 {
                let (date_str, time_str) = grid_text(grid_s, level.medium_s < 3600, options);
                let text_x = line_x + 4.0;
                let text_color = if info.ctx.style().visuals.dark_mode {
                    Color32::from(Rgba::from_white_alpha(
                        (text_alpha * alpha_multiplier * 2.0).min(1.0),
                    ))
                } else {
                    Color32::from(Rgba::from_black_alpha(
                        (text_alpha * alpha_multiplier * 2.0).min(1.0),
                    ))
                };

                info.painter.fonts(|f| {
                    let mut row_y = fixed_timeline_y;
                    if let Some(d) = date_str {
                        shapes.push(egui::Shape::text(
                            f,
                            pos2(text_x, row_y),
                            Align2::LEFT_TOP,
                            &d,
                            info.font_id.clone(),
                            text_color,
                        ));
                        row_y += info.text_height;
                    }
                    if let Some(t) = time_str {
                        shapes.push(egui::Shape::text(
                            f,
                            pos2(text_x, row_y),
                            Align2::LEFT_TOP,
                            &t,
                            info.font_id.clone(),
                            text_color,
                        ));
                    }
                });
            }
        }

        grid_s += level.step_s;
    }

    shapes
}

pub(super) fn paint_timeline_text_on_top(
    info: &Info,
    options: &Options,
    fixed_timeline_y: f32,
    gutter_width: f32,
) {
    let max_lines = info.usable_width() / 13.0;
    let level = pick_grid_level(options, max_lines);

    let theme_colors = get_theme_colors(&info.ctx.style());

    let alpha_multiplier = if info.ctx.style().visuals.dark_mode { 0.3 } else { 0.8 };

    let num_tiny_lines = options.canvas_width_s / (level.step_s as f32);
    let zoom_factor = remap_clamp(num_tiny_lines, (0.1 * max_lines)..=max_lines, 1.0..=0.0);

    let header_h = timeline_header_height(options, info.usable_width(), info.text_height);
    let bg_rect = Rect::from_min_size(
        pos2(info.canvas.min.x + gutter_width, fixed_timeline_y),
        egui::vec2(info.usable_width(), header_h),
    );
    info.painter
        .rect_filled(bg_rect, 0.0, theme_colors.background_timeline);

    let timeline_text = paint_timeline_text(
        info,
        info.canvas,
        options,
        level,
        fixed_timeline_y,
        alpha_multiplier,
        zoom_factor,
    );

    for shape in timeline_text {
        info.painter.add(shape);
    }
}

pub(super) fn paint_timeline(
    info: &Info,
    canvas: Rect,
    options: &Options,
    _start_s: i64,
    _gutter_width: f32,
) -> Vec<egui::Shape> {
    let mut shapes = vec![];
    let theme_colors = get_theme_colors(&info.ctx.style());

    let alpha_multiplier = if info.ctx.style().visuals.dark_mode { 0.3 } else { 0.8 };

    let max_lines = info.usable_width() / 13.0;
    let level = pick_grid_level(options, max_lines);

    let num_tiny_lines = options.canvas_width_s / (level.step_s as f32);
    let zoom_factor = remap_clamp(num_tiny_lines, (0.1 * max_lines)..=max_lines, 1.0..=0.0);
    let zoom_factor = zoom_factor * zoom_factor;
    let big_alpha   = remap_clamp(zoom_factor, 0.0..=1.0, 0.5..=1.0);
    let medium_alpha = remap_clamp(zoom_factor, 0.0..=1.0, 0.1..=0.5);
    let tiny_alpha  = remap_clamp(zoom_factor, 0.0..=1.0, 0.0..=0.1);

    // Start at the first grid line at or before the visible left edge.
    let visible_left_s = info.start_s
        - ((options.sideways_pan_in_points / info.usable_width()) * options.canvas_width_s) as i64;
    let mut grid_s = (visible_left_s / level.step_s) * level.step_s;

    loop {
        let line_x = info.point_from_s(options, grid_s);

        if line_x > canvas.max.x {
            break;
        }

        if canvas.min.x <= line_x {
            let big_line    = grid_s % level.big_s    == 0;
            let medium_line = grid_s % level.medium_s == 0;

            let line_alpha = if big_line {
                big_alpha
            } else if medium_line {
                medium_alpha
            } else {
                tiny_alpha
            };

            shapes.push(egui::Shape::line_segment(
                [pos2(line_x, canvas.min.y), pos2(line_x, canvas.max.y)],
                Stroke::new(
                    1.0,
                    theme_colors
                        .line
                        .linear_multiply(line_alpha * alpha_multiplier),
                ),
            ));
        }

        grid_s += level.step_s;
    }

    shapes
}

pub(super) fn paint_current_time_line(
    info: &Info,
    options: &Options,
    canvas: Rect,
    _gutter_width: f32,
) -> egui::Shape {
    let current_time = chrono::Utc::now().timestamp();
    let line_x = info.point_from_s(options, current_time);

    egui::Shape::line_segment(
        [pos2(line_x, canvas.min.y), pos2(line_x, canvas.max.y)],
        Stroke::new(2.0, options.now_line_color),
    )
}

fn grid_text(ts: i64, split: bool, options: &Options) -> (Option<String>, Option<String>) {
    if ts == 0 {
        return (Some("N/A".to_string()), None);
    }
    let Some(dt) = DateTime::from_timestamp(ts, 0) else {
        return (Some("Invalid timestamp".to_string()), None);
    };
    let local = dt.with_timezone(&Local);

    let mut date_parts = vec![];
    if options.show_year { date_parts.push(local.format("%Y").to_string()); }
    if options.show_month { date_parts.push(local.format("%m").to_string()); }
    if options.show_day { date_parts.push(local.format("%d").to_string()); }
    let date_str = (!date_parts.is_empty()).then(|| date_parts.join("-"));

    let mut time_parts = vec![];
    if options.show_hour { time_parts.push(local.format("%H").to_string()); }
    if options.show_minute { time_parts.push(local.format("%M").to_string()); }
    if options.show_second { time_parts.push(local.format("%S").to_string()); }
    let time_str = (!time_parts.is_empty()).then(|| time_parts.join(":"));

    if split {
        (date_str, time_str)
    } else {
        let combined = match (date_str, time_str) {
            (Some(d), Some(t)) => Some(format!("{d} {t}")),
            (Some(d), None) => Some(d),
            (None, Some(t)) => Some(t),
            (None, None) => None,
        };
        (combined, None)
    }
}
