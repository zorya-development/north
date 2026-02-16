use std::f64::consts::PI;

use leptos::prelude::*;

use north_ui::{Icon, IconKind};

/// Circular task checkbox with subtask progress arc.
///
/// - No subtasks, unchecked: muted border circle
/// - No subtasks, checked: filled accent circle with check icon
/// - Has subtasks, unchecked: muted border circle with clockwise progress arc
///   starting from 12 o'clock (e.g. 3/12 subtasks = arc from 12 to 3)
/// - Has subtasks, checked: fully filled accent circle with check icon
#[component]
pub fn TaskCheckboxView(
    is_completed: Memo<bool>,
    /// None = no subtasks, Some((completed, total)) = has subtasks
    progress: Memo<Option<(i64, i64)>>,
    on_toggle: Callback<()>,
) -> impl IntoView {
    let size = 16.0_f64;
    let r = 6.5_f64;
    let cx = size / 2.0;
    let cy = size / 2.0;
    let stroke_w = 2;

    view! {
        <button
            on:click=move |_| on_toggle.run(())
            class="flex-shrink-0 group"
            aria-label=move || {
                if is_completed.get() {
                    "Mark task incomplete"
                } else {
                    "Complete task"
                }
            }
        >
            {move || {
                let completed = is_completed.get();
                let prog = progress.get();

                if completed {
                    // Filled circle with check icon
                    view! {
                        <div class="w-4 h-4 rounded-full bg-accent \
                                    group-hover:bg-accent-hover flex \
                                    items-center justify-center \
                                    transition-all duration-200">
                            <Icon
                                kind=IconKind::Check
                                class="w-3 h-3 text-on-accent"
                            />
                        </div>
                    }
                    .into_any()
                } else {
                    match prog {
                        Some((done, total)) if total > 0 && done > 0 => {
                            let fraction = done as f64 / total as f64;
                            let arc = arc_path(cx, cy, r, fraction);
                            view! {
                                <svg
                                    width=size.to_string()
                                    height=size.to_string()
                                    viewBox=format!(
                                        "0 0 {} {}", size, size,
                                    )
                                    class="transition-all duration-200"
                                >
                                    // Background circle (muted)
                                    <circle
                                        cx=cx.to_string()
                                        cy=cy.to_string()
                                        r=r.to_string()
                                        fill="none"
                                        stroke="var(--text-secondary)"
                                        stroke-width=stroke_w.to_string()
                                        opacity="0.5"
                                        class="group-hover:stroke-[var(--accent)] \
                                               transition-[stroke] duration-200"
                                    />
                                    // Progress arc
                                    <path
                                        d=arc
                                        fill="none"
                                        stroke="var(--accent)"
                                        stroke-width=stroke_w.to_string()
                                        stroke-linecap="round"
                                    />
                                </svg>
                            }
                            .into_any()
                        }
                        _ => {
                            // Simple muted border circle (no subtasks or 0 done)
                            view! {
                                <svg
                                    width=size.to_string()
                                    height=size.to_string()
                                    viewBox=format!(
                                        "0 0 {} {}", size, size,
                                    )
                                    class="transition-all duration-200"
                                >
                                    <circle
                                        cx=cx.to_string()
                                        cy=cy.to_string()
                                        r=r.to_string()
                                        fill="none"
                                        stroke="var(--text-secondary)"
                                        stroke-width=stroke_w.to_string()
                                        opacity="0.5"
                                        class="group-hover:stroke-[var(--accent)] \
                                               group-hover:opacity-100 \
                                               transition-all duration-200"
                                    />
                                </svg>
                            }
                            .into_any()
                        }
                    }
                }
            }}
        </button>
    }
}

/// Build an SVG arc path from 12-o'clock clockwise for `fraction` of the circle.
fn arc_path(cx: f64, cy: f64, r: f64, fraction: f64) -> String {
    let fraction = fraction.clamp(0.0, 1.0);
    if fraction <= 0.0 {
        return String::new();
    }
    if fraction >= 1.0 {
        // Full circle — two semicircles
        return format!(
            "M {cx} {sy} A {r} {r} 0 1 1 {cx} {ey} A {r} {r} 0 1 1 {cx} {sy}",
            cx = cx,
            sy = cy - r,
            ey = cy + r,
            r = r,
        );
    }

    // Start at 12 o'clock (top center)
    let start_x = cx;
    let start_y = cy - r;

    // Angle in radians, starting from -PI/2 (12 o'clock), going clockwise
    let angle = 2.0 * PI * fraction - PI / 2.0;
    let end_x = cx + r * angle.cos();
    let end_y = cy + r * angle.sin();

    let large_arc = if fraction > 0.5 { 1 } else { 0 };

    format!(
        "M {sx} {sy} A {r} {r} 0 {la} 1 {ex} {ey}",
        sx = start_x,
        sy = start_y,
        r = r,
        la = large_arc,
        ex = end_x,
        ey = end_y,
    )
}
