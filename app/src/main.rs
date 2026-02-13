use gloo_net::http::Request;
use gloo_timers::callback::Interval;
use serde::Deserialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

const WORKER_BASE: &str = "https://daily-wonder-worker.mikegyver.workers.dev";

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct DailyWonderCard {
    fun_fact: String,
    thought: String,
    kindness: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct Meta {
    mode: String,            // "daily" | "surprise"
    day_utc: String,         // YYYY-MM-DD
    next_reset_utc_ms: i64,  // epoch ms (UTC midnight)
    ttl_seconds: i64,        // seconds remaining on edge cache
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct CardResponse {
    card: DailyWonderCard,
    meta: Meta,
}

fn now_ms() -> i64 {
    // Works in WASM
    js_sys::Date::now() as i64
}

fn format_hms(total_seconds: i64) -> String {
    let s = total_seconds.max(0);
    let h = s / 3600;
    let m = (s % 3600) / 60;
    let sec = s % 60;
    format!("{:02}:{:02}:{:02}", h, m, sec)
}

#[function_component(App)]
fn app() -> Html {
    let data = use_state(|| None::<CardResponse>);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    // animation / UX states
    let anim_key = use_state(|| 0u32);
    let is_fading = use_state(|| false);

    // countdown (seconds)
    let countdown = use_state(|| 0_i64);

    let fetch_endpoint = |path: &'static str| {
        let data = data.clone();
        let error = error.clone();
        let loading = loading.clone();
        let anim_key = anim_key.clone();
        let is_fading = is_fading.clone();

        Callback::from(move |_| {
            let data = data.clone();
            let error = error.clone();
            let loading = loading.clone();
            let anim_key = anim_key.clone();
            let is_fading = is_fading.clone();

            // start animation
            is_fading.set(true);
            loading.set(true);
            error.set(None);

            spawn_local(async move {
                let url = format!("{WORKER_BASE}{path}");

                let result = Request::get(&url)
                    .header("Accept", "application/json")
                    .send()
                    .await;

                match result {
                    Ok(response) => {
                        let status = response.status();
                        match response.json::<CardResponse>().await {
                            Ok(parsed) => {
                                data.set(Some(parsed));
                                anim_key.set(*anim_key + 1);
                            }
                            Err(err) => error.set(Some(format!(
                                "Failed to parse card (HTTP {status}). {err}"
                            ))),
                        }
                    }
                    Err(err) => error.set(Some(format!("Request failed: {err}"))),
                }

                loading.set(false);

                // end fade shortly after content change
                {
                    let is_fading = is_fading.clone();
                    spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(180).await;
                        is_fading.set(false);
                    });
                }
            });
        })
    };

    let fetch_daily = fetch_endpoint("/card");
    let fetch_surprise = fetch_endpoint("/surprise");

    // Initial fetch on mount
    {
        let fetch_daily = fetch_daily.clone();
        use_effect_with((), move |_| {
            fetch_daily.emit(());
            || ()
        });
    }

    // Countdown tick: uses meta.next_reset_utc_ms when present
    {
        let data = data.clone();
        let countdown = countdown.clone();

        use_effect_with(data.clone(), move |_| {
            // set immediately
            if let Some(resp) = &*data {
                let secs = (resp.meta.next_reset_utc_ms - now_ms()) / 1000;
                countdown.set(secs.max(0));
            } else {
                countdown.set(0);
            }

            let handle = Interval::new(1000, move || {
                if let Some(resp) = &*data {
                    let secs = (resp.meta.next_reset_utc_ms - now_ms()) / 1000;
                    countdown.set(secs.max(0));
                } else {
                    countdown.set(0);
                }
            });

            move || drop(handle)
        });
    }

    let on_refresh = {
        let fetch_daily = fetch_daily.clone();
        Callback::from(move |_e: MouseEvent| fetch_daily.emit(()))
    };

    let on_surprise = {
        let fetch_surprise = fetch_surprise.clone();
        Callback::from(move |_e: MouseEvent| fetch_surprise.emit(()))
    };

    let pill_text = if let Some(resp) = &*data {
        let hms = format_hms(*countdown);
        format!("Next card in {hms} • UTC reset • mode: {}", resp.meta.mode)
    } else {
        "Next card: —".to_string()
    };

    let status_block = if *loading {
        html! {
            <div class="statusWrap">
                <span class="spinner" aria-hidden="true"></span>
                <span class="statusText">{"Loading..."}</span>
            </div>
        }
    } else if let Some(message) = &*error {
        html! { <p class="status error">{message}</p> }
    } else {
        html! { <p class="status subtle">{"Tip: Daily is cached. Surprise is one-off."}</p> }
    };

    let content = if let Some(resp) = &*data {
        let card = &resp.card;
        let fade_class = if *is_fading { "card fade" } else { "card" };

        html! {
            <div class={fade_class} key={(*anim_key).to_string()}>
                <div class="cardTop">
                    <h2>{"🌟 Fun Fact"}</h2>
                    <span class="pill">{ resp.meta.day_utc.clone() }</span>
                </div>

                <p>{card.fun_fact.clone()}</p>

                <h2>{"💡 Thought"}</h2>
                <p>{card.thought.clone()}</p>

                <h2>{"❤️ Tiny Kindness Idea"}</h2>
                <p>{card.kindness.clone()}</p>
            </div>
        }
    } else {
        html! {}
    };

    html! {
        <div class="container">
            <header>
                <h1>{"Daily Wonder Card"}</h1>
                <p>{"A tiny spark of curiosity + kindness. Fresh each day."}</p>

                <div class="actions">
                    <button class="refresh" onclick={on_refresh} disabled={*loading}>
                        { if *loading { "Refreshing..." } else { "Refresh (Daily)" } }
                    </button>

                    <button class="surprise" onclick={on_surprise} disabled={*loading}>
                        {"🎲 Surprise me"}
                    </button>

                    <span class="pill countdown">{ pill_text }</span>
                </div>
            </header>

            {status_block}
            {content}
        </div>
    }
}

fn main() -> Result<(), JsValue> {
    yew::Renderer::<App>::new().render();
    Ok(())
}