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
    let (count, set_count) = create_signal(cx, 0);
    // let on_click = move |_| set_count.update(|count| *count += 1);

    view! { cx,
        <div class="p-2">
            <h1>"Welcome to Chat WebRTC! Enter Server Multiaddress:"</h1>
            <UncontrolledComponent />
        </div>
    }
}
