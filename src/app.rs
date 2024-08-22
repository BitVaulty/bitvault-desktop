use std::sync::Arc;

use leptos::{leptos_dom::ev::SubmitEvent, *};
use leptos_meta::*;
use leptos_router::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

use crate::icons;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Serialize, Deserialize)]
struct GreetArgs<'a> {
    name: &'a str,
}

#[derive(Clone, Debug, Default)]
pub enum WalletState {
    #[default]
    New,
    Creating,
    Restoring,
    // Unlocked,
    // Locked,
}

// Define a struct to hold the global state
#[derive(Clone, Debug, Default)]
pub struct AppState {
    pub user_pin: Option<String>,
    pub wallet_state: WalletState,
}

// Create a type alias for a thread-safe, shared reference to the state
pub type SharedAppState = Arc<RwLock<AppState>>;

// Function to create and provide the global state
fn provide_app_state() -> SharedAppState {
    let app_state = AppState::default();
    let shared_state = Arc::new(RwLock::new(app_state));

    // Provide the state to Leptos context
    provide_context(shared_state.clone());

    shared_state
}

// Helper function to get the app state from context
pub fn use_app_state() -> SharedAppState {
    use_context::<SharedAppState>().expect("AppState not found in context")
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

    // Initialize and provide the global state
    let _state = provide_app_state();

    view! {
        <Stylesheet id="leptos" href="/style/output.css"/>
        <Link rel="shortcut icon" type_="image/ico" href="/favicon.ico"/>
        <Router>
            <Routes>
                <Route path="/" view=Home/>
                <Route path="/disclaimer" view=Disclaimer />
                <Route path="/wallet" view=Wallet />
                <Route path="/pin-choice" view=PinChoice />
                <Route path="/seed" view=Seed />
                <Route path="/seed-verify" view=SeedVerify />
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    let set_wallet_state_creating = move || {
        use_app_state().write().wallet_state = WalletState::Creating;
    };

    let set_wallet_state_restoring = move || {
        use_app_state().write().wallet_state = WalletState::Restoring;
    };

    view! {
        <div class="flex justify-center items-center min-h-screen bg-white dark:bg-gray-900">
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
                        <A href="/disclaimer" on:click=move |_| set_wallet_state_creating()>
                            <div class="w-full max-w-sm h-14 px-5 py-5 bg-amber-500 rounded flex-col justify-center items-center gap-2.5 flex">
                                <div class="text-white text-xl font-semibold font-['Inter'] leading-snug">"Create a new wallet"</div>
                            </div>
                        </A>
                        <A href="/disclaimer" on:click=move |_| set_wallet_state_restoring()>
                            <div class="h-14 justify-center items-center gap-2.5 inline-flex">
                                <div class="w-full max-w-sm text-center text-amber-500 text-xl font-normal font-['Inter'] leading-7">"Restore existing wallet"</div>
                            </div>
                        </A>
                    </div>
                    <div class="text-center text-neutral-500 dark:text-neutral-300 text-base font-normal font-['Inter'] leading-tight mt-8">"Your fortress against physical attacks and hacks, by employing time-delayed transactions and a multisig convenience service to shield your assets. Fully open source and non-custodial."</div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Disclaimer() -> impl IntoView {
    let (checkbox1, set_checkbox1) = create_signal(false);
    let (checkbox2, set_checkbox2) = create_signal(false);

    let both_checked = move || checkbox1.get() && checkbox2.get();

    view! {
        <div class="flex justify-center items-center min-h-screen bg-white dark:bg-gray-900">
            <div class="w-full max-w-md p-6 bg-white dark:bg-gray-800 rounded-lg shadow-lg">
                <div class="flex items-center mb-6">
                    <A href="/">
                        <div class="p-2 rounded justify-start items-center flex cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700">
                            <div class="w-5 h-5 mr-2">
                                <icons::CaretLeft />
                            </div>
                            <div class="text-black dark:text-white text-lg font-semibold font-['Inter']">Back</div>
                        </div>
                    </A>
                </div>
                <div class="flex flex-col items-center gap-7">
                    <div class="flex flex-col items-center gap-6">
                        <div class="p-3.5 bg-green-500 rounded-full">
                            <div class="w-7 h-7">
                                <icons::Wallet />
                            </div>
                        </div>
                        <div class="text-center">
                            <h2 class="text-black dark:text-white text-3xl font-semibold font-['Inter'] leading-10">
                                "Two things you" <br/>"must understand"
                            </h2>
                        </div>
                    </div>
                    <div class="w-full space-y-4">
                        <div class="self-stretch flex-col justify-start items-start flex">
                            <div class="h-px relative bg-neutral-200"></div>
                            <div class="self-stretch py-3.5 justify-start items-center gap-2.5 inline-flex">
                                <div class="grow shrink basis-0 self-stretch flex-col justify-start items-start gap-1 inline-flex">
                                    <div class="self-stretch text-neutral-500 text-lg font-normal font-['Inter'] leading-relaxed cursor-pointer"
                                         on:click=move |_| set_checkbox1.update(|v| *v = true)
                                    >"With bitcoin, you are your own bank. No one else has access to "<br/>"your private keys."</div>
                                </div>
                                <div class="justify-start items-center flex">
                                    <input
                                        type="checkbox"
                                        id="checkbox1"
                                        class="w-6 h-6 text-amber-500 bg-gray-100 border-gray-300 rounded focus:ring-amber-500 dark:focus:ring-amber-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600 transition-all duration-200 ease-in-out"
                                        on:click=move |_| set_checkbox1.update(|v| *v = !*v)
                                        prop:checked=checkbox1
                                    />
                                </div>
                            </div>
                            <div class="h-px relative bg-neutral-200"></div>
                            <div class="self-stretch py-3.5 justify-start items-center gap-2.5 inline-flex">
                                <div class="grow shrink basis-0 self-stretch flex-col justify-start items-start gap-1 inline-flex">
                                    <div class="self-stretch text-neutral-500 text-lg font-normal font-['Inter'] leading-relaxed cursor-pointer"
                                         on:click=move |_| set_checkbox2.update(|v| *v = true)
                                    >"If you lose access to this app, any backups that exist, your bitcoin cannot be recovered."</div>
                                </div>
                                <div class="justify-start items-center flex">
                                    <input
                                        type="checkbox"
                                        id="checkbox2"
                                        class="w-6 h-6 text-amber-500 bg-gray-100 border-gray-300 rounded focus:ring-amber-500 dark:focus:ring-amber-600 dark:ring-offset-gray-800 focus:ring-2 dark:bg-gray-700 dark:border-gray-600 transition-all duration-200 ease-in-out"
                                        on:click=move |_| set_checkbox2.update(|v| *v = !*v)
                                        prop:checked=checkbox2
                                    />
                                </div>
                            </div>
                            <div class="h-px relative bg-neutral-200"></div>
                        </div>
                    </div>
                    <button
                        class=move || {
                            let base_classes = "w-full mt-6 px-5 py-3.5 text-white rounded transition duration-300 ease-in-out";
                            if both_checked() {
                                format!("{} bg-amber-500 hover:bg-amber-600", base_classes)
                            } else {
                                format!("{} bg-gray-300 cursor-not-allowed", base_classes)
                            }
                        }
                        disabled=move || !both_checked()
                    >
                        <A href="/pin-choice" class="w-full h-full flex items-center justify-center">
                            <span class={move || {
                                if both_checked() {
                                    "w-full h-full flex items-center justify-center text-lg font-semibold font-['Inter'] text-white"
                                } else {
                                    "w-full h-full flex items-center justify-center text-lg font-semibold font-['Inter'] text-gray-500 dark:text-gray-400"
                                }
                            }}>
                                "Next"
                            </span>
                        </A>
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn PinChoice() -> impl IntoView {
    use leptos::*;
    use web_sys::KeyboardEvent;

    let (pin, set_pin) = create_signal(Vec::new());
    let (confirm_pin, set_confirm_pin) = create_signal(Vec::new());
    let (is_confirming, set_is_confirming) = create_signal(false);
    let (error_message, set_error_message) = create_signal(Option::None);
    let input_ref = create_node_ref::<html::Input>();

    let check_pins = move || {
        if pin.with(|p| p.clone()) == confirm_pin.with(|p| p.clone()) {
            // Pins match, navigate to the next screen
            let state = use_app_state();
            let mut state = state.write();
            state.user_pin = Some(pin.with(|p| p.iter().collect::<String>()));
            let window = web_sys::window().unwrap();
            let _ = window.location().set_href("/seed");
        } else {
            // Pins don't match, reset confirmation and show error message
            set_confirm_pin.update(|p| p.clear());
            set_is_confirming.set(false);
            set_error_message.set(Some("Incorrect PIN. Please try again.".to_string()));
            if let Some(input) = input_ref.get() {
                set_pin.update(|p| p.clear());
                let _ = input.focus();
            }
        }
    };

    let handle_key_press = move |event: KeyboardEvent| {
        let key = event.key();
        match key.as_str() {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                if !is_confirming.get() && pin.with(|p| p.len()) < 6 {
                    set_pin.update(|p| p.push(key.chars().next().unwrap()));
                    if pin.with(|p| p.len()) == 6 {
                        set_is_confirming.set(true);
                        set_error_message.set(None);
                    }
                } else if is_confirming.get() && confirm_pin.with(|p| p.len()) < 6 {
                    set_confirm_pin.update(|p| p.push(key.chars().next().unwrap()));
                    if confirm_pin.with(|p| p.len()) == 6 {
                        check_pins();
                    }
                }
            }
            "Backspace" => {
                if !is_confirming.get() {
                    set_pin.update(|p| {
                        p.pop();
                    });
                } else {
                    set_confirm_pin.update(|p| {
                        p.pop();
                    });
                }
            }
            _ => {}
        }
    };

    let add_digit = move |digit: char| {
        if !is_confirming.get() && pin.with(|p| p.len()) < 6 {
            set_pin.update(|p| p.push(digit));
            if pin.with(|p| p.len()) == 6 {
                set_is_confirming.set(true);
                set_error_message.set(None);
            }
        } else if is_confirming.get() && confirm_pin.with(|p| p.len()) < 6 {
            set_confirm_pin.update(|p| p.push(digit));
            if confirm_pin.with(|p| p.len()) == 6 {
                check_pins();
            }
        }
    };

    let remove_digit = move |_| {
        if !is_confirming.get() {
            set_pin.update(|p| {
                p.pop();
            });
        } else {
            set_confirm_pin.update(|p| {
                p.pop();
            });
        }
    };

    let pin_display = move || {
        let current_pin = if is_confirming.get() {
            confirm_pin
        } else {
            pin
        };
        (0..6)
            .map(|i| {
                let filled = current_pin.with(|p| p.len() > i);
                view! {
                    <div class=move || {
                        if filled {
                            "w-5 h-5 bg-amber-500 rounded-full"
                        } else {
                            "w-5 h-5 rounded-full border-2 border-neutral-400"
                        }
                    }></div>
                }
            })
            .collect::<Vec<_>>()
    };

    // Focus the input field when the component mounts
    create_effect(move |_| {
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    });

    let focus_input = move |_| {
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    };

    view! {
        <div class="flex justify-center items-center min-h-screen bg-white dark:bg-gray-900"
        on:click=focus_input>
            <div class="w-full max-w-md p-6 bg-white dark:bg-gray-800 rounded-lg shadow-lg">
                <div class="flex items-center mb-6">
                    <A href="/disclaimer">
                        <div class="p-2 rounded justify-start items-center flex cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700">
                            <div class="w-5 h-5 mr-2">
                                <icons::CaretLeft />
                            </div>
                            <div class="text-black dark:text-white text-lg font-semibold font-['Inter']">Back</div>
                        </div>
                    </A>
                </div>
                <div class="w-full bg-gray-800 rounded-3xl flex-col justify-center items-start inline-flex">
                    <input
                        type="text"
                        _ref=input_ref
                        on:keydown=handle_key_press
                        style="position: absolute; opacity: 0; pointer-events: none;"
                    />
                    <div class="self-stretch grow shrink basis-0 px-5 pt-14 pb-6 flex-col justify-center items-center gap-24 inline-flex">
                        <div class="self-stretch flex-col justify-start items-center gap-12 flex">
                            <div class="self-stretch flex-col justify-start items-center gap-2.5 flex">
                                <div class="text-center text-white text-xl font-semibold font-['Inter'] leading-7">
                                    {move || if is_confirming.get() {
                                        "Confirm your 6-digit PIN"
                                    } else {
                                        "Choose a 6-digit PIN"
                                    }}
                                </div>
                                <div class="w-80 text-center text-gray-400 text-lg font-normal font-['Inter'] leading-relaxed">
                                    {move || if is_confirming.get() {
                                        "Please enter your PIN again to confirm."
                                    } else {
                                        "PIN entry will be required for wallet access and transactions. Write it down as it cannot be recovered."
                                    }}
                                </div>
                            </div>
                            <div class="justify-start items-start gap-2.5 inline-flex">
                                {pin_display}
                            </div>
                            {move || error_message.get().map(|msg| view! {
                                <div class="w-full text-center text-red-500 text-sm font-normal font-['Inter'] leading-tight mt-4">
                                    {msg}
                                </div>
                            })}
                        </div>
                    </div>
                    <div class="self-stretch pb-12 flex-col justify-start items-start gap-6 inline-flex">
                        {["123", "456", "789"].into_iter().map(|row| {
                            view! {
                                <div class="self-stretch justify-center items-start inline-flex">
                                    {row.chars().map(|digit| {
                                        let digit_clone = digit;
                                        view! {
                                            <button
                                                class="grow shrink basis-0 h-16 flex-col justify-center items-center inline-flex"
                                                on:click=move |_| {
                                                    add_digit(digit_clone);
                                                    let _ = input_ref.get().unwrap().focus();
                                                }
                                            >
                                                <div class="self-stretch text-center text-white text-2xl font-normal font-['Inter'] leading-loose">
                                                    {digit}
                                                </div>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                        <div class="self-stretch justify-center items-start inline-flex">
                            <div class="grow shrink basis-0 h-16 opacity-0"></div>
                            <button
                                class="grow shrink basis-0 h-16 flex-col justify-center items-center inline-flex"
                                on:click=move |_| {
                                    add_digit('0');
                                    let _ = input_ref.get().unwrap().focus();
                                }
                            >
                                <div class="self-stretch text-center text-white text-2xl font-normal font-['Inter'] leading-loose">
                                    "0"
                                </div>
                            </button>
                            <button
                                class="grow shrink basis-0 h-16 flex-col justify-center items-center inline-flex"
                                on:click=move |_| {
                                    remove_digit(());
                                    let _ = input_ref.get().unwrap().focus();
                                }
                            >
                                <div class="self-stretch text-center text-white text-2xl font-normal font-['Inter'] leading-loose flex justify-center items-center">
                                    <div class="w-6 h-6">
                                        <icons::ArrowLeft />
                                    </div>
                                </div>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

use anyhow::Result;
use bip39::{Language, Mnemonic};
use zeroize::Zeroize;

fn new_seed_handler() -> Result<String> {
    let mut entropy = [0u8; 16];
    getrandom::getrandom(&mut entropy)?;
    let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)?;
    entropy.zeroize();
    Ok(mnemonic.to_string())
}
fn new_seed_fn(_args: &str) -> Option<String> {
    new_seed_handler().ok()
}

#[component]
fn Seed() -> impl IntoView {
    let (new_seed, set_new_seed) = create_signal(vec![]);

    spawn_local(async move {
        let args = to_value("null").unwrap();
        // let result = invoke("new_seed", args).await.as_string();
        let result = new_seed_fn("null");
        if let Some(result_string) = result {
            let seed_words: Vec<String> =
                result_string.split_whitespace().map(String::from).collect();
            set_new_seed.set(seed_words);
        } else {
            // Handle the error case, e.g., log it or set an error state
            log::error!("Failed to get new seed");
        }
    });

    let seed_words = [
        "gloom", "police", "month", "stamp", "viable", "claim", "hospital", "heart", "alcohol",
        "off", "ocean", "ghost",
    ];

    set_new_seed.set(seed_words.iter().map(|s| s.to_string()).collect());

    view! {
        <div class="flex justify-center items-center min-h-screen bg-white dark:bg-gray-900">
            <div class="w-full max-w-md p-6 bg-white dark:bg-gray-800 rounded-lg shadow-lg">
                <div class="flex items-center mb-6">
                    <A href="/pin-choice">
                        <div class="p-2 rounded justify-start items-center flex cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-700">
                            <div class="w-5 h-5 mr-2">
                                <icons::CaretLeft />
                            </div>
                            <div class="text-black dark:text-white text-lg font-semibold font-['Inter']">Back</div>
                        </div>
                    </A>
                </div>
                <div class="flex flex-col items-center gap-6">
                    <div class="text-center">
                        <h2 class="text-black dark:text-white text-xl font-semibold font-['Inter'] mb-2">
                            "This is your recovery phrase"
                        </h2>
                        <p class="text-neutral-500 dark:text-neutral-400 text-lg font-normal font-['Inter']">
                            "Make sure to write it down as shown here. You have to verify this later."
                        </p>
                    </div>
                    <div class="grid grid-cols-2 gap-3 w-full">
                        {move || new_seed.get().into_iter().enumerate().map(|(index, word)| {
                            view! {
                                <div class="bg-gray-200 dark:bg-gray-700 rounded-full flex items-center">
                                    <div class="h-11 px-3 flex items-center">
                                        <span class="text-black dark:text-white text-lg font-semibold font-['Inter']">
                                            {index + 1}
                                        </span>
                                    </div>
                                    <div class="w-0.5 h-11 bg-white dark:bg-gray-600"></div>
                                    <div class="flex-grow px-3">
                                        <span class="text-black dark:text-white text-lg font-semibold font-['Inter']">
                                            {word}
                                        </span>
                                    </div>
                                </div>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                    <button class="w-full h-full px-5 py-3.5 bg-amber-500 hover:bg-amber-600 rounded text-white text-lg font-semibold font-['Inter']">
                        <A href="/seed-verify" class="w-full h-full flex items-center justify-center hover:text-white">
                            "Verify"
                        </A>
                    </button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn SeedVerify() -> impl IntoView {
    view! {
        "TODO"
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
