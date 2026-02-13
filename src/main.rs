use gloo::render::{request_animation_frame, AnimationFrame};
use serde::{Deserialize, Serialize};
use std::{cell::Cell, rc::Rc};
use yew::prelude::*;

const SNAPSHOT_KEY: &str = "orbit_of_ideas_snapshots_v1";

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Idea {
    id: u32,
    title: String,
    notes: String,
    impact: f64,
    urgency: f64,
    effort: f64,
    phase: f64,
    hue: f64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct Link {
    from: u32,
    to: u32,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct SimulatorParams {
    base_radius: f64,
    gravity_gain: f64,
    urgency_gain: f64,
    speed_gain: f64,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct ExportBundle {
    ideas: Vec<Idea>,
    links: Vec<Link>,
    sim: SimulatorParams,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct SnapshotEntry {
    name: String,
    data: ExportBundle,
}

#[derive(Clone, PartialEq)]
struct NewIdeaForm {
    title: String,
    notes: String,
    impact: f64,
    urgency: f64,
    effort: f64,
}

#[derive(Clone, PartialEq)]
struct Star {
    x: f64,
    y: f64,
    r: f64,
    alpha: f64,
}

#[derive(Clone, Copy)]
struct Metrics {
    ideas: f64,
    avg_radius: f64,
    avg_speed: f64,
    impact: f64,
    urgency: f64,
    effort: f64,
}

fn load_snapshots() -> Vec<SnapshotEntry> {
    let Some(window) = web_sys::window() else {
        return Vec::new();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return Vec::new();
    };
    let Ok(Some(raw)) = storage.get_item(SNAPSHOT_KEY) else {
        return Vec::new();
    };
    serde_json::from_str::<Vec<SnapshotEntry>>(&raw).unwrap_or_default()
}

fn store_snapshots(snapshots: &[SnapshotEntry]) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return;
    };
    if let Ok(raw) = serde_json::to_string(snapshots) {
        let _ = storage.set_item(SNAPSHOT_KEY, &raw);
    }
}

fn metrics_from(bundle: &ExportBundle) -> Metrics {
    if bundle.ideas.is_empty() {
        return Metrics {
            ideas: 0.0,
            avg_radius: 0.0,
            avg_speed: 0.0,
            impact: 0.0,
            urgency: 0.0,
            effort: 0.0,
        };
    }

    let count = bundle.ideas.len() as f64;
    let mut radius_sum = 0.0;
    let mut speed_sum = 0.0;
    let mut impact_sum = 0.0;
    let mut urgency_sum = 0.0;
    let mut effort_sum = 0.0;

    for idea in &bundle.ideas {
        let radius = orbit_radius(idea, &bundle.sim);
        let speed = orbit_speed(idea, radius, &bundle.sim);
        radius_sum += radius;
        speed_sum += speed;
        impact_sum += idea.impact;
        urgency_sum += idea.urgency;
        effort_sum += idea.effort;
    }

    Metrics {
        ideas: count,
        avg_radius: radius_sum / count,
        avg_speed: speed_sum / count,
        impact: impact_sum / count,
        urgency: urgency_sum / count,
        effort: effort_sum / count,
    }
}

fn delta_class(value: f64) -> &'static str {
    if value > 0.0 {
        "delta-pos"
    } else if value < 0.0 {
        "delta-neg"
    } else {
        "delta-flat"
    }
}

fn truncated_label(title: &str, max_chars: usize) -> String {
    if title.chars().count() <= max_chars {
        title.to_owned()
    } else {
        format!("{}…", title.chars().take(max_chars).collect::<String>())
    }
}

fn hue_from_id(id: u32) -> f64 {
    ((id as f64 * 53.0 + 170.0) % 360.0).abs()
}

fn orbit_radius(idea: &Idea, sim: &SimulatorParams) -> f64 {
    (sim.base_radius + idea.effort * 1.4
        - idea.impact * sim.gravity_gain * 0.85
        - idea.urgency * sim.urgency_gain * 0.9)
        .clamp(62.0, 300.0)
}

fn orbit_speed(idea: &Idea, radius: f64, sim: &SimulatorParams) -> f64 {
    (sim.speed_gain * 0.45 + (idea.urgency / 100.0) * 1.2 + (190.0 / radius) * 0.6
        - (idea.effort / 100.0) * 0.75)
        .clamp(0.1, 3.2)
}

fn sat_xy(idea: &Idea, sim: &SimulatorParams) -> (f64, f64, f64, f64) {
    let radius = orbit_radius(idea, sim);
    let speed = orbit_speed(idea, radius, sim);
    let x = 360.0 + radius * idea.phase.cos();
    let y = 260.0 + radius * idea.phase.sin();
    (x, y, radius, speed)
}

fn generate_stars(count: usize) -> Vec<Star> {
    let mut seed = 0x1234_5678_9abc_def0_u64;
    (0..count)
        .map(|_| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let x = ((seed >> 12) % 720) as f64;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let y = ((seed >> 10) % 520) as f64;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let r = 0.6 + (((seed >> 8) % 18) as f64 / 10.0);
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let alpha = 0.18 + (((seed >> 6) % 55) as f64 / 100.0);
            Star { x, y, r, alpha }
        })
        .collect()
}

#[function_component(App)]
fn app() -> Html {
    let ideas = use_state(|| {
        vec![
            Idea {
                id: 1,
                title: "Ship MVP".into(),
                notes: "Focus release on 2 wow moments and stable onboarding.".into(),
                impact: 85.0,
                urgency: 72.0,
                effort: 40.0,
                phase: 0.4,
                hue: hue_from_id(1),
            },
            Idea {
                id: 2,
                title: "Creator Growth Loop".into(),
                notes: "Invite > share > remix orbit.".into(),
                impact: 75.0,
                urgency: 64.0,
                effort: 58.0,
                phase: 1.9,
                hue: hue_from_id(2),
            },
            Idea {
                id: 3,
                title: "Studio Narrative".into(),
                notes: "Clarify mission language and visual cadence.".into(),
                impact: 63.0,
                urgency: 48.0,
                effort: 28.0,
                phase: 3.7,
                hue: hue_from_id(3),
            },
        ]
    });
    let links = use_state(|| vec![Link { from: 1, to: 2 }, Link { from: 2, to: 3 }]);
    let snapshots = use_state(load_snapshots);
    let selected_id = use_state(|| Some(1_u32));
    let next_id = use_state(|| 4_u32);
    let link_target = use_state(|| 2_u32);
    let sim = use_state(|| SimulatorParams {
        base_radius: 182.0,
        gravity_gain: 1.0,
        urgency_gain: 0.8,
        speed_gain: 1.0,
    });
    let json_blob = use_state(String::new);
    let snapshot_name = use_state(String::new);
    let compare_a = use_state(|| 0_usize);
    let compare_b = use_state(|| 0_usize);

    let form = use_state(|| NewIdeaForm {
        title: String::new(),
        notes: String::new(),
        impact: 50.0,
        urgency: 50.0,
        effort: 50.0,
    });

    let stars = use_state(|| generate_stars(78));
    let last_time = use_mut_ref(js_sys::Date::now);
    let raf_handle = use_mut_ref(|| Option::<AnimationFrame>::None);

    {
        let ideas = ideas.clone();
        let sim = sim.clone();
        let last_time = last_time.clone();
        let raf_handle = raf_handle.clone();

        use_effect_with((), move |_| {
            let running = Rc::new(Cell::new(true));

            fn tick(
                ideas: UseStateHandle<Vec<Idea>>,
                sim: UseStateHandle<SimulatorParams>,
                last_time: UseMutRefHandle<f64>,
                raf_handle: UseMutRefHandle<Option<AnimationFrame>>,
                running: Rc<Cell<bool>>,
            ) {
                let ideas_in = ideas.clone();
                let sim_in = sim.clone();
                let last_time_in = last_time.clone();
                let raf_in = raf_handle.clone();
                let running_in = running.clone();

                *raf_handle.borrow_mut() = Some(request_animation_frame(move |t| {
                    if !running_in.get() {
                        return;
                    }

                    let mut prev = last_time_in.borrow_mut();
                    let dt = ((t - *prev) / 1000.0).clamp(0.0, 0.05);
                    *prev = t;

                    let sim_now = (*sim_in).clone();
                    ideas_in.set(
                        ideas_in
                            .iter()
                            .cloned()
                            .map(|mut idea| {
                                let radius = orbit_radius(&idea, &sim_now);
                                let speed = orbit_speed(&idea, radius, &sim_now);
                                idea.phase += speed * dt;
                                idea
                            })
                            .collect(),
                    );

                    tick(ideas_in, sim_in, last_time_in, raf_in, running_in);
                }));
            }

            tick(ideas, sim, last_time, raf_handle, running.clone());

            move || {
                running.set(false);
                let _ = raf_handle.borrow_mut().take();
            }
        });
    }

    let selected_idea = selected_id
        .as_ref()
        .and_then(|id| ideas.iter().find(|i| i.id == *id).cloned());

    let current_bundle = ExportBundle {
        ideas: (*ideas).clone(),
        links: (*links).clone(),
        sim: (*sim).clone(),
    };

    let a_snapshot = snapshots.get(*compare_a).cloned();
    let b_snapshot = snapshots.get(*compare_b).cloned();
    let compare_html = if let (Some(a), Some(b)) = (a_snapshot, b_snapshot) {
        let a_metrics = metrics_from(&a.data);
        let b_metrics = metrics_from(&b.data);

        let delta_ideas = b_metrics.ideas - a_metrics.ideas;
        let delta_radius = b_metrics.avg_radius - a_metrics.avg_radius;
        let delta_speed = b_metrics.avg_speed - a_metrics.avg_speed;
        let delta_impact = b_metrics.impact - a_metrics.impact;
        let delta_urgency = b_metrics.urgency - a_metrics.urgency;
        let delta_effort = b_metrics.effort - a_metrics.effort;

        html! {
            <>
                <div class="muted">{format!("Comparing {} → {}", a.name, b.name)}</div>
                <div class="delta-grid">
                    <span class={classes!("delta", delta_class(delta_ideas))}>{format!("Ideas {:+.0}", delta_ideas)}</span>
                    <span class={classes!("delta", delta_class(delta_radius))}>{format!("Radius {:+.1}", delta_radius)}</span>
                    <span class={classes!("delta", delta_class(delta_speed))}>{format!("Speed {:+.2}", delta_speed)}</span>
                    <span class={classes!("delta", delta_class(delta_impact))}>{format!("Impact {:+.1}", delta_impact)}</span>
                    <span class={classes!("delta", delta_class(delta_urgency))}>{format!("Urgency {:+.1}", delta_urgency)}</span>
                    <span class={classes!("delta", delta_class(delta_effort))}>{format!("Effort {:+.1}", delta_effort)}</span>
                </div>
            </>
        }
    } else {
        html! { <div class="muted">{"Save at least two snapshots to compare."}</div> }
    };

    let update_selected_metric = {
        let ideas = ideas.clone();
        let selected_id = selected_id.clone();
        move |metric: &'static str, value: f64| {
            if let Some(active_id) = *selected_id {
                ideas.set(
                    ideas
                        .iter()
                        .cloned()
                        .map(|mut i| {
                            if i.id == active_id {
                                match metric {
                                    "impact" => i.impact = value,
                                    "urgency" => i.urgency = value,
                                    "effort" => i.effort = value,
                                    _ => {}
                                }
                            }
                            i
                        })
                        .collect(),
                );
            }
        }
    };

    let on_delete = {
        let ideas = ideas.clone();
        let links = links.clone();
        let selected_id = selected_id.clone();
        Callback::from(move |_| {
            if let Some(active_id) = *selected_id {
                let mut remaining: Vec<Idea> = ideas
                    .iter()
                    .filter(|x| x.id != active_id)
                    .cloned()
                    .collect();
                remaining.sort_by_key(|x| x.id);
                let next_selected = remaining.first().map(|x| x.id);
                selected_id.set(next_selected);
                ideas.set(remaining);
                links.set(
                    links
                        .iter()
                        .filter(|l| l.from != active_id && l.to != active_id)
                        .cloned()
                        .collect(),
                );
            }
        })
    };

    let add_idea = {
        let ideas = ideas.clone();
        let form = form.clone();
        let next_id = next_id.clone();
        let selected_id = selected_id.clone();
        Callback::from(move |_| {
            if form.title.trim().is_empty() {
                return;
            }
            let id = *next_id;
            let mut new_ideas = (*ideas).clone();
            new_ideas.push(Idea {
                id,
                title: form.title.trim().to_string(),
                notes: form.notes.trim().to_string(),
                impact: form.impact,
                urgency: form.urgency,
                effort: form.effort,
                phase: id as f64 * 0.78,
                hue: hue_from_id(id),
            });
            ideas.set(new_ideas);
            next_id.set(id + 1);
            selected_id.set(Some(id));
            form.set(NewIdeaForm {
                title: String::new(),
                notes: String::new(),
                impact: 50.0,
                urgency: 50.0,
                effort: 50.0,
            });
        })
    };

    let add_link = {
        let links = links.clone();
        let selected_id = selected_id.clone();
        let link_target = link_target.clone();
        Callback::from(move |_| {
            if let Some(from) = *selected_id {
                let to = *link_target;
                if from == to {
                    return;
                }
                if links.iter().any(|l| l.from == from && l.to == to) {
                    return;
                }
                let mut next_links = (*links).clone();
                next_links.push(Link { from, to });
                links.set(next_links);
            }
        })
    };

    let remove_links_from_selected = {
        let links = links.clone();
        let selected_id = selected_id.clone();
        Callback::from(move |_| {
            if let Some(from) = *selected_id {
                links.set(links.iter().filter(|l| l.from != from).cloned().collect());
            }
        })
    };

    let export_json = {
        let current_bundle = current_bundle.clone();
        let json_blob = json_blob.clone();
        Callback::from(move |_| {
            if let Ok(value) = serde_json::to_string_pretty(&current_bundle) {
                json_blob.set(value);
            }
        })
    };

    let import_json = {
        let ideas = ideas.clone();
        let links = links.clone();
        let sim = sim.clone();
        let selected_id = selected_id.clone();
        let next_id = next_id.clone();
        let json_blob = json_blob.clone();
        Callback::from(move |_| {
            if let Ok(payload) = serde_json::from_str::<ExportBundle>(&json_blob) {
                let next = payload.ideas.iter().map(|i| i.id).max().unwrap_or(0) + 1;
                let select = payload.ideas.first().map(|x| x.id);
                ideas.set(payload.ideas);
                links.set(payload.links);
                sim.set(payload.sim);
                selected_id.set(select);
                next_id.set(next);
            }
        })
    };

    let save_snapshot = {
        let snapshots = snapshots.clone();
        let snapshot_name = snapshot_name.clone();
        let current_bundle = current_bundle.clone();
        Callback::from(move |_| {
            let name = snapshot_name.trim().to_string();
            if name.is_empty() {
                return;
            }
            let mut next = (*snapshots).clone();
            if let Some(existing) = next.iter_mut().find(|s| s.name == name) {
                existing.data = current_bundle.clone();
            } else {
                next.push(SnapshotEntry {
                    name,
                    data: current_bundle.clone(),
                });
            }
            store_snapshots(&next);
            snapshots.set(next);
        })
    };

    let load_snapshot = {
        let snapshots = snapshots.clone();
        let compare_b = compare_b.clone();
        let ideas = ideas.clone();
        let links = links.clone();
        let sim = sim.clone();
        let selected_id = selected_id.clone();
        let next_id = next_id.clone();
        Callback::from(move |_| {
            if let Some(entry) = snapshots.get(*compare_b) {
                ideas.set(entry.data.ideas.clone());
                links.set(entry.data.links.clone());
                sim.set(entry.data.sim.clone());
                selected_id.set(entry.data.ideas.first().map(|x| x.id));
                next_id.set(entry.data.ideas.iter().map(|i| i.id).max().unwrap_or(0) + 1);
            }
        })
    };

    let auto_stabilize = {
        let ideas = ideas.clone();
        let sim = sim.clone();
        Callback::from(move |_| {
            if ideas.is_empty() {
                return;
            }
            let mut radii: Vec<f64> = ideas.iter().map(|i| orbit_radius(i, &sim)).collect();
            radii.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mean = radii.iter().sum::<f64>() / radii.len() as f64;
            let avg_gap = if radii.len() > 1 {
                (radii[radii.len() - 1] - radii[0]) / (radii.len() - 1) as f64
            } else {
                24.0
            };
            let min_gap = radii
                .windows(2)
                .map(|w| w[1] - w[0])
                .fold(999.0, |acc, g| acc.min(g));

            let mut tuned = (*sim).clone();
            let spread_factor = (26.0 / avg_gap).clamp(0.7, 1.4);
            tuned.gravity_gain = (tuned.gravity_gain * spread_factor).clamp(0.25, 2.0);
            tuned.urgency_gain = (tuned.urgency_gain * spread_factor).clamp(0.25, 2.0);
            tuned.base_radius = (tuned.base_radius + (170.0 - mean) * 0.35).clamp(120.0, 260.0);
            tuned.speed_gain = if min_gap < 14.0 {
                (tuned.speed_gain * 0.92).clamp(0.2, 2.0)
            } else {
                (tuned.speed_gain * 1.03).clamp(0.2, 2.0)
            };
            sim.set(tuned);
        })
    };

    html! {
        <main class="app">
            <h1 class="title">{"Orbit of Ideas"}</h1>
            <div class="layout">
                <section class="card">
                    <div class="orbit-wrap">
                        <svg viewBox="0 0 720 520" width="100%" role="img" aria-label="Mission idea orbit simulator">
                            <defs>
                                <radialGradient id="missionGlow" cx="50%" cy="50%" r="50%">
                                    <stop offset="0%" stop-color="#c4ecff" stop-opacity="0.9" />
                                    <stop offset="100%" stop-color="#4cb7ff" stop-opacity="0" />
                                </radialGradient>
                            </defs>

                            { for stars.iter().map(|s| html! {
                                <circle cx={s.x.to_string()} cy={s.y.to_string()} r={s.r.to_string()} fill="white" opacity={s.alpha.to_string()} />
                            }) }

                            <circle cx="360" cy="260" r="88" fill="url(#missionGlow)" />
                            <circle cx="360" cy="260" r="16" fill="#d6f5ff" opacity="0.95" />
                            <text x="360" y="295" text-anchor="middle" font-size="16" fill="#dff1ff" opacity="0.9">{"MISSION"}</text>

                            { for links.iter().filter_map(|link| {
                                let from = ideas.iter().find(|i| i.id == link.from)?;
                                let to = ideas.iter().find(|i| i.id == link.to)?;
                                let (x1, y1, _, _) = sat_xy(from, &sim);
                                let (x2, y2, _, _) = sat_xy(to, &sim);
                                let dist = ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
                                let collision = dist < 36.0;
                                Some(html! {
                                    <line
                                        x1={x1.to_string()}
                                        y1={y1.to_string()}
                                        x2={x2.to_string()}
                                        y2={y2.to_string()}
                                        stroke={if collision {"#ff7f97"} else {"#82c9ff"}}
                                        stroke-opacity={if collision {"0.95"} else {"0.55"}}
                                        stroke-width={if collision {"2.2"} else {"1.4"}}
                                        stroke-dasharray={if collision {"3 5"} else {"6 6"}}
                                    />
                                })
                            }) }

                            { for ideas.iter().map(|idea| {
                                let (x, y, radius, speed) = sat_xy(idea, &sim);
                                let selected = Some(idea.id) == *selected_id;
                                let stroke = if selected { "0.78" } else { "0.3" };
                                let sat_r = if selected { 9.4 } else { 7.1 };
                                let sat_alpha = if selected { 0.98 } else { 0.85 };
                                let label_x = x + 12.0;
                                let label_y = y - 12.0;
                                let title = truncated_label(&idea.title, 18);
                                let ring_click = {
                                    let selected_id = selected_id.clone();
                                    let id = idea.id;
                                    Callback::from(move |_| selected_id.set(Some(id)))
                                };

                                html! {
                                    <g>
                                        <circle cx="360" cy="260" r={radius.to_string()} fill="none" stroke="#9ec9ff" stroke-opacity={stroke} stroke-width={if selected {"1.9"} else {"1.0"}} onclick={ring_click.clone()} />
                                        <circle cx={x.to_string()} cy={y.to_string()} r={sat_r.to_string()} fill={format!("hsl({:.0} 90% 66%)", idea.hue)} opacity={sat_alpha.to_string()} stroke="white" stroke-opacity={if selected {"0.9"} else {"0.4"}} stroke-width={if selected {"1.8"} else {"1.0"}} onclick={ring_click.clone()} />
                                        <text x={label_x.to_string()} y={label_y.to_string()} fill="#cfe6ff" font-size={if selected {"13"} else {"12"}} opacity="0.92">{title}</text>
                                        <text x={label_x.to_string()} y={(label_y + 13.0).to_string()} fill="#9fb7dc" font-size="10">{format!("r:{:.0} s:{:.2}", radius, speed)}</text>
                                    </g>
                                }
                            }) }
                        </svg>
                    </div>
                </section>

                <section class="card controls">
                    <div class="section">
                        <h3>{"Telemetry"}</h3>
                        {
                            if let Some(sel) = &selected_idea {
                                html! {
                                    <>
                                        <div class="pill">{format!("Selected: {}", sel.title)}</div>
                                        <div class="slider-row">
                                            <label><span>{"Impact"}</span><span>{format!("{:.0}", sel.impact)}</span></label>
                                            <input type="range" min="0" max="100" value={sel.impact.to_string()} oninput={{
                                                let updater = update_selected_metric.clone();
                                                Callback::from(move |e: InputEvent| updater("impact", e.target_unchecked_into::<web_sys::HtmlInputElement>().value_as_number()))
                                            }} />
                                        </div>
                                        <div class="slider-row">
                                            <label><span>{"Urgency"}</span><span>{format!("{:.0}", sel.urgency)}</span></label>
                                            <input type="range" min="0" max="100" value={sel.urgency.to_string()} oninput={{
                                                let updater = update_selected_metric.clone();
                                                Callback::from(move |e: InputEvent| updater("urgency", e.target_unchecked_into::<web_sys::HtmlInputElement>().value_as_number()))
                                            }} />
                                        </div>
                                        <div class="slider-row">
                                            <label><span>{"Effort"}</span><span>{format!("{:.0}", sel.effort)}</span></label>
                                            <input type="range" min="0" max="100" value={sel.effort.to_string()} oninput={{
                                                let updater = update_selected_metric.clone();
                                                Callback::from(move |e: InputEvent| updater("effort", e.target_unchecked_into::<web_sys::HtmlInputElement>().value_as_number()))
                                            }} />
                                        </div>
                                        <div class="muted">{format!("Notes: {}", if sel.notes.is_empty() {"(none)"} else {&sel.notes})}</div>
                                        <div class="btn-row" style="margin-top:8px;">
                                            <button class="danger" onclick={on_delete.clone()}>{"Delete idea"}</button>
                                        </div>
                                    </>
                                }
                            } else {
                                html! { <div class="muted">{"No idea selected."}</div> }
                            }
                        }
                    </div>

                    <div class="section">
                        <h3>{"Simulator knobs"}</h3>
                        <div class="slider-row">
                            <label><span>{"Base radius"}</span><span>{format!("{:.0}", sim.base_radius)}</span></label>
                            <input type="range" min="120" max="260" value={sim.base_radius.to_string()} oninput={{
                                let sim = sim.clone();
                                Callback::from(move |e: InputEvent| {
                                    let mut s = (*sim).clone();
                                    s.base_radius = e.target_unchecked_into::<web_sys::HtmlInputElement>().value_as_number();
                                    sim.set(s);
                                })
                            }} />
                        </div>
                        <div class="slider-row">
                            <label><span>{"Gravity gain"}</span><span>{format!("{:.2}", sim.gravity_gain)}</span></label>
                            <input type="range" min="0.2" max="2.0" step="0.01" value={sim.gravity_gain.to_string()} oninput={{
                                let sim = sim.clone();
                                Callback::from(move |e: InputEvent| {
                                    let mut s = (*sim).clone();
                                    s.gravity_gain = e.target_unchecked_into::<web_sys::HtmlInputElement>().value_as_number();
                                    sim.set(s);
                                })
                            }} />
                        </div>
                        <div class="slider-row">
                            <label><span>{"Urgency gain"}</span><span>{format!("{:.2}", sim.urgency_gain)}</span></label>
                            <input type="range" min="0.2" max="2.0" step="0.01" value={sim.urgency_gain.to_string()} oninput={{
                                let sim = sim.clone();
                                Callback::from(move |e: InputEvent| {
                                    let mut s = (*sim).clone();
                                    s.urgency_gain = e.target_unchecked_into::<web_sys::HtmlInputElement>().value_as_number();
                                    sim.set(s);
                                })
                            }} />
                        </div>
                        <div class="slider-row">
                            <label><span>{"Speed gain"}</span><span>{format!("{:.2}", sim.speed_gain)}</span></label>
                            <input type="range" min="0.2" max="2.0" step="0.01" value={sim.speed_gain.to_string()} oninput={{
                                let sim = sim.clone();
                                Callback::from(move |e: InputEvent| {
                                    let mut s = (*sim).clone();
                                    s.speed_gain = e.target_unchecked_into::<web_sys::HtmlInputElement>().value_as_number();
                                    sim.set(s);
                                })
                            }} />
                        </div>
                        <div class="btn-row">
                            <button onclick={auto_stabilize}>{"Auto-stabilize spacing"}</button>
                        </div>
                    </div>

                    <div class="section">
                        <h3>{"Dependencies"}</h3>
                        <div class="slider-row">
                            <label>{"Target idea"}</label>
                            <select onchange={{
                                let link_target = link_target.clone();
                                Callback::from(move |e: Event| {
                                    let v = e.target_unchecked_into::<web_sys::HtmlSelectElement>().value();
                                    if let Ok(id) = v.parse::<u32>() {
                                        link_target.set(id);
                                    }
                                })
                            }}>
                                { for ideas.iter().map(|idea| html! {
                                    <option value={idea.id.to_string()} selected={*link_target == idea.id}>{&idea.title}</option>
                                }) }
                            </select>
                        </div>
                        <div class="btn-row">
                            <button onclick={add_link}>{"Add link from selected"}</button>
                            <button class="danger" onclick={remove_links_from_selected}>{"Clear selected links"}</button>
                        </div>
                    </div>

                    <div class="section">
                        <h3>{"Scenario snapshots"}</h3>
                        <div class="slider-row">
                            <label>{"Snapshot name"}</label>
                            <input type="text" value={(*snapshot_name).clone()} placeholder="e.g. Q2 aggressive launch" oninput={{
                                let snapshot_name = snapshot_name.clone();
                                Callback::from(move |e: InputEvent| {
                                    snapshot_name.set(e.target_unchecked_into::<web_sys::HtmlInputElement>().value());
                                })
                            }} />
                        </div>
                        <div class="btn-row">
                            <button onclick={save_snapshot.clone()}>{"Save / update snapshot"}</button>
                            <button onclick={load_snapshot}>{"Load snapshot B"}</button>
                        </div>
                        <div class="slider-row" style="margin-top:8px;">
                            <label>{"Scenario A"}</label>
                            <select onchange={{
                                let compare_a = compare_a.clone();
                                Callback::from(move |e: Event| {
                                    if let Ok(v) = e.target_unchecked_into::<web_sys::HtmlSelectElement>().value().parse::<usize>() {
                                        compare_a.set(v);
                                    }
                                })
                            }}>
                                { for snapshots.iter().enumerate().map(|(idx, snap)| html! {
                                    <option value={idx.to_string()} selected={*compare_a == idx}>{&snap.name}</option>
                                }) }
                            </select>
                        </div>
                        <div class="slider-row">
                            <label>{"Scenario B"}</label>
                            <select onchange={{
                                let compare_b = compare_b.clone();
                                Callback::from(move |e: Event| {
                                    if let Ok(v) = e.target_unchecked_into::<web_sys::HtmlSelectElement>().value().parse::<usize>() {
                                        compare_b.set(v);
                                    }
                                })
                            }}>
                                { for snapshots.iter().enumerate().map(|(idx, snap)| html! {
                                    <option value={idx.to_string()} selected={*compare_b == idx}>{&snap.name}</option>
                                }) }
                            </select>
                        </div>
                        {compare_html}
                    </div>

                    <div class="section">
                        <h3>{"Add idea"}</h3>
                        <div class="slider-row">
                            <label>{"Title"}</label>
                            <input type="text" value={form.title.clone()} oninput={{
                                let form = form.clone();
                                Callback::from(move |e: InputEvent| {
                                    let mut f = (*form).clone();
                                    f.title = e.target_unchecked_into::<web_sys::HtmlInputElement>().value();
                                    form.set(f);
                                })
                            }} placeholder="e.g. Launch soundtrack drop" />
                        </div>
                        <div class="slider-row">
                            <label>{"Notes"}</label>
                            <textarea value={form.notes.clone()} oninput={{
                                let form = form.clone();
                                Callback::from(move |e: InputEvent| {
                                    let mut f = (*form).clone();
                                    f.notes = e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value();
                                    form.set(f);
                                })
                            }} />
                        </div>
                        {
                            for ["impact", "urgency", "effort"].iter().map(|key| {
                                let value = match *key {
                                    "impact" => form.impact,
                                    "urgency" => form.urgency,
                                    _ => form.effort,
                                };
                                html! {
                                    <div class="slider-row">
                                        <label><span>{key.to_uppercase()}</span><span>{format!("{:.0}", value)}</span></label>
                                        <input type="range" min="0" max="100" value={value.to_string()} oninput={{
                                            let form = form.clone();
                                            let key = key.to_string();
                                            Callback::from(move |e: InputEvent| {
                                                let mut f = (*form).clone();
                                                let num = e.target_unchecked_into::<web_sys::HtmlInputElement>().value_as_number();
                                                match key.as_str() {
                                                    "impact" => f.impact = num,
                                                    "urgency" => f.urgency = num,
                                                    _ => f.effort = num,
                                                }
                                                form.set(f);
                                            })
                                        }} />
                                    </div>
                                }
                            })
                        }
                        <div class="btn-row">
                            <button onclick={add_idea}>{"Add idea to orbit"}</button>
                        </div>
                    </div>

                    <div class="section">
                        <h3>{"Import / Export JSON"}</h3>
                        <textarea value={(*json_blob).clone()} oninput={{
                            let json_blob = json_blob.clone();
                            Callback::from(move |e: InputEvent| {
                                json_blob.set(e.target_unchecked_into::<web_sys::HtmlTextAreaElement>().value());
                            })
                        }} />
                        <div class="btn-row" style="margin-top:8px;">
                            <button onclick={export_json}>{"Export"}</button>
                            <button onclick={import_json}>{"Import"}</button>
                        </div>
                    </div>

                    <div class="section">
                        <h3>{"Ideas"}</h3>
                        <div class="idea-list">
                            { for ideas.iter().map(|idea| {
                                let is_active = Some(idea.id) == *selected_id;
                                let select = {
                                    let selected_id = selected_id.clone();
                                    let id = idea.id;
                                    Callback::from(move |_| selected_id.set(Some(id)))
                                };
                                html! {
                                    <button onclick={select} class={classes!("idea-item", is_active.then_some("active"))}>
                                        <div>{&idea.title}</div>
                                        <div class="muted">{format!("I {:.0} • U {:.0} • E {:.0}", idea.impact, idea.urgency, idea.effort)}</div>
                                    </button>
                                }
                            }) }
                        </div>
                    </div>
                </section>
            </div>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
