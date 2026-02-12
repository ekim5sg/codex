use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen::JsValue;
use yew::prelude::*;

const API_URL: &str = "https://daily-wonder-worker.mikegyver.workers.dev/card";

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct DailyWonderCard {
    // Worker returns { day, fun_fact, thought, kindness }
    // We ignore day by making it optional.
    #[serde(default)]
    day: Option<String>,

    fun_fact: String,
    thought: String,
    kindness: String,
}

#[function_component(App)]
fn app() -> Html {
    let card = use_state(|| None::<DailyWonderCard>);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    // animation / UX states
    let anim_key = use_state(|| 0u32);     // forces re-mount of card for animation
    let is_fading = use_state(|| false);   // CSS class toggle

    let fetch_card = {
        let card = card.clone();
        let error = error.clone();
        let loading = loading.clone();
        let anim_key = anim_key.clone();
        let is_fading = is_fading.clone();

        Callback::from(move |_| {
            let card = card.clone();
            let error = error.clone();
            let loading = loading.clone();
            let anim_key = anim_key.clone();
            let is_fading = is_fading.clone();

            // start animation
            is_fading.set(true);
            loading.set(true);
            error.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let result = Request::get(API_URL)
                    .header("Accept", "application/json")
                    .send()
                    .await;

                match result {
                    Ok(response) => {
                        let status = response.status();
                        match response.json::<DailyWonderCard>().await {
                            Ok(data) => {
                                card.set(Some(data));
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
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(180).await;
                        is_fading.set(false);
                    });
                }
            });
        })
    };

    // Fetch once on mount
    {
        let fetch_card = fetch_card.clone();
        use_effect_with((), move |_| {
            fetch_card.emit(());
            || ()
        });
    }

    // onclick wrapper
    let on_refresh = {
        let fetch_card = fetch_card.clone();
        Callback::from(move |_e: MouseEvent| fetch_card.emit(()))
    };

    let status_block = if *loading {
        html! {
            <div class="statusWrap">
                <span class="spinner" aria-hidden="true"></span>
                <span class="statusText">{"Loading today's wonder..."}</span>
            </div>
        }
    } else if let Some(message) = &*error {
        html! { <p class="status error">{message}</p> }
    } else {
        html! { <p class="status subtle">{"Tip: Refresh anytime. The card is the same all day."}</p> }
    };

    let card_block = if let Some(card) = &*card {
        let fade_class = if *is_fading { "card fade" } else { "card" };
        html! {
            <div class={fade_class} key={(*anim_key).to_string()}>
                <div class="cardTop">
                    <h2>{"🌟 Fun Fact"}</h2>
                    if let Some(day) = &card.day {
                        <span class="pill">{day.clone()}</span>
                    }
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
                        { if *loading { "Refreshing..." } else { "Refresh" } }
                    </button>
                </div>
            </header>

            {status_block}
            {card_block}
        </div>
    }
}

fn main() -> Result<(), JsValue> {
    yew::Renderer::<App>::new().render();
    Ok(())
}