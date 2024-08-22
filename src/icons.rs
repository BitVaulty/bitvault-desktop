use leptos::{component, view, IntoView};

#[component]
pub fn CaretLeft() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path fill-rule="evenodd" d="M14.601 4.47a.75.75 0 010 1.06l-6.364 6.364a.25.25 0 000 .354l6.364 6.364a.75.75 0 01-1.06 1.06l-6.364-6.364a1.75 1.75 0 010-2.474L13.54 4.47a.75.75 0 011.06 0z" clip-rule="evenodd"></path></svg>
    }
}

#[component]
pub fn Wallet() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path fill-rule="evenodd" d="M12 8a2 2 0 012-2h4a2 2 0 012 2v8a2 2 0 01-2 2h-4a2 2 0 01-2-2V8zm2-1a1 1 0 00-1 1v8a1 1 0 001 1h4a1 1 0 001-1V8a1 1 0 00-1-1h-4z" clip-rule="evenodd"></path><path fill-rule="evenodd" d="M5.5 6A1.5 1.5 0 004 7.5v9A1.5 1.5 0 005.5 18h10a1.5 1.5 0 001.5-1.5v-9A1.5 1.5 0 0015.5 6h-10zm2 7.5a1.5 1.5 0 100-3 1.5 1.5 0 000 3z" clip-rule="evenodd"></path></svg>
    }
}

#[component]
pub fn ArrowLeft() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor"><path fill-rule="evenodd" d="M2.461 12a.75.75 0 01.75-.75l17.79.012a.75.75 0 11-.002 1.5L3.21 12.75a.75.75 0 01-.749-.75z" clip-rule="evenodd"></path><path fill-rule="evenodd" d="M10.517 4.47a.75.75 0 01.001 1.06L4.06 12l6.458 6.47a.75.75 0 01-1.061 1.06l-6.988-7a.75.75 0 010-1.06l6.988-7a.75.75 0 011.06 0z" clip-rule="evenodd"></path></svg>
    }
}
