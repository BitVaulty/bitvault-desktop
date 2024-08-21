use leptos::{leptos_dom::ev::SubmitEvent, *};
use leptos_meta::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[component]
pub fn Greet() -> impl IntoView {
    let (name, set_name) = create_signal(String::new());
    let (greet_msg, set_greet_msg) = create_signal(String::new());

    let update_name = move |ev| {
        let v = event_target_value(&ev);
        set_name.set(v);
    };

    let greet = move |ev: SubmitEvent| {
        ev.prevent_default();
        spawn_local(async move {
            let name = name.get_untracked();
            if name.is_empty() {
                return;
            }

            let args = to_value(&GreetArgs { name: &name }).unwrap();
            // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
            let new_msg = invoke("greet", args).await.as_string().unwrap();
            set_greet_msg.set(new_msg);
        });
    };

    view! {
        <main class="container">
            <form class="row" on:submit=greet>
                <input
                    id="greet-input"
                    placeholder="Enter a name..."
                    on:input=update_name
                />
                <button type="submit">"Greet"</button>
            </form>

            <p><b>{ move || greet_msg.get() }</b></p>
        </main>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/style/output.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Router>
            <Routes>
                <Route path="/" view=Home/>
                <Route path="/wallet" view=Wallet />
            </Routes>

            // <nav>
            //     <A href="/">"BitVaulty"</A>
            //     <A href="/wallet">"Wallet"</A>
            //     <A href="/settings">"Settings"</A>
            //     // <button on:click=move |_| {
            //     //     set_logged_in.update(|n| *n = !*n)
            //     // }>{move || if logged_in.get() { "Log Out" } else { "Log In" }}</button>
            // </nav>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    view! {
        <div class="container mx-auto px-4">
            <div class="rounded-3xl flex-col justify-center items-center mx-auto max-w-md">
                <div class="self-stretch grow shrink basis-0 px-5 pt-14 pb-12 flex-col justify-start items-center inline-flex">
                    <div class="self-stretch grow shrink basis-0 flex-col justify-start items-center gap-12 flex">
                        <div class="flex-col justify-start items-center gap-7 flex">
                            <div class="w-24 h-24 relative bg-amber-500 rounded-full">
                                <img src="/public/bitcoin.png" />
                            </div>
                            <div class="flex-col justify-start items-center gap-2.5 flex">
                                <div class="w-80 text-center text-black dark:text-white text-4xl font-semibold font-['Inter'] leading-10">"BitVaulty"</div>
                                <div class="w-80 text-center text-neutral-500 dark:text-neutral-300 text-2xl font-normal font-['Inter'] leading-loose">"Be Your Own Bank, Safely."</div>
                            </div>
                        </div>
                    </div>
                    <div class="self-stretch mt-12 flex-col justify-center items-center gap-2.5 flex">
                        <div class="w-full max-w-sm h-14 px-5 py-5 bg-amber-500 rounded flex-col justify-center items-center gap-2.5 flex">
                            <div class="text-white text-xl font-semibold font-['Inter'] leading-snug">Create a new wallet</div>
                        </div>
                        <div class="h-14 justify-center items-center gap-2.5 inline-flex">
                            <div class="w-full max-w-sm text-center text-amber-500 text-xl font-normal font-['Inter'] leading-7">Restore existing wallet</div>
                        </div>
                    </div>
                    <div class="text-center text-neutral-500 dark:text-neutral-300 text-base font-normal font-['Inter'] leading-tight mt-8">"Your fortress against physical attacks and hacks, by employing time-delayed transactions and a multisig convenience service to shield your assets. Fully open source and non-custodial."</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Wallet() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div class="my-0 mx-auto max-w-3xl text-center">
            <h2 class="p-6 text-4xl">"Wallet"</h2>
            <p class="px-10 pb-10 text-left">"Tailwind will scan your Rust files for Tailwind class names and compile them into a CSS file."</p>
            <button
                class="bg-amber-600 hover:bg-sky-700 px-5 py-3 text-white rounded-lg"
                on:click=move |_| set_count.update(|count| *count += 1)
            >
                "Something's here | "
                {move || if count.get() == 0 {
                    "Click me!".to_string()
                } else {
                    count.get().to_string()
                }}
                " | Some more text"
            </button>
        </div>
    }
}
