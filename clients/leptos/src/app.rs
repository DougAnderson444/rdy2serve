use crate::components::inputs::*;
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

/// Our App Compoennts
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="output.css"/>

        // sets the document title
        <Title text="Welcome to Leptos"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes>
                    <Route path="" view=|cx| view! { cx, <HomePage/> }/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage(cx: Scope) -> impl IntoView {
    // Creates a reactive value to update the button
    let (addr, set_addr) = create_signal(cx, "Unset Yet".to_string());

    view! { cx,
        <div class="p-2">
            <h1 class="text-xl font-semibold mt-3 mb-2">
                "Welcome to Rust Chat WebRTC LibP2P!"
            </h1>
            <h2 class="font-semibold mt-3 mb-2">
                "Enter Server Multiaddress:"
            </h2>
            <UncontrolledComponent on_set=set_addr />

            <div class="border rounded p-2 m-2">
                <p>"Connecting to Address: "<span class="break-words font-mono text-xs">{addr}</span> </p>
            </div>
        </div>
    }
}
