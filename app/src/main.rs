use gloo_net::http::Request;
use serde::Deserialize;
use wasm_bindgen::JsValue;
use yew::prelude::*;

#[derive(Debug, Clone, Deserialize, PartialEq)]
struct DailyWonderCard {
    fun_fact: String,
    thought: String,
    kindness: String,
}

#[function_component(App)]
fn app() -> Html {
    let card = use_state(|| None::<DailyWonderCard>);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| false);

    let fetch_card = {
        let card = card.clone();
        let error = error.clone();
        let loading = loading.clone();
        Callback::from(move |_| {
            let card = card.clone();
            let error = error.clone();
            let loading = loading.clone();
            loading.set(true);
            error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let result = Request::get("/card")
                    .header("Accept", "application/json")
                    .send()
                    .await;

                match result {
                    Ok(response) => match response.json::<DailyWonderCard>().await {
                        Ok(data) => card.set(Some(data)),
                        Err(err) => error.set(Some(format!("Failed to parse card: {err}"))),
                    },
                    Err(err) => error.set(Some(format!("Request failed: {err}"))),
                }

                loading.set(false);
            });
        })
    };

    {
        let fetch_card = fetch_card.clone();
        use_effect_with_deps(
            move |_| {
                fetch_card.emit(());
                || ()
            },
            (),
        );
    }

    let content = if *loading {
        html! { <p class="status">{"Loading today's wonder..."}</p> }
    } else if let Some(card) = &*card {
        html! {
            <div class="card">
                <h2>{"🌟 Fun Fact"}</h2>
                <p>{card.fun_fact.clone()}</p>
                <h2>{"💡 Thought"}</h2>
                <p>{card.thought.clone()}</p>
                <h2>{"❤️ Tiny Kindness Idea"}</h2>
                <p>{card.kindness.clone()}</p>
            </div>
        }
    } else if let Some(message) = &*error {
        html! { <p class="status error">{message}</p> }
    } else {
        html! { <p class="status">{"Click refresh to load a card."}</p> }
    };

    html! {
        <div class="container">
            <header>
                <h1>{"Daily Wonder Card"}</h1>
                <p>{"Refresh to get a new spark of curiosity and kindness."}</p>
            </header>
            <button class="refresh" onclick={fetch_card} disabled={*loading}>
                {"Refresh"}
            </button>
            {content}
        </div>
    }
}

fn main() -> Result<(), JsValue> {
    yew::Renderer::<App>::new().render();
    Ok(())
}
