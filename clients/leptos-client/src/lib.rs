use leptos::html::Input;
use leptos::*;
// use leptos::{html::Input, *};

// const ENTER_KEY: u32 = 13;

/// A simple chat component.
///
#[component]
pub fn SimpleChat(
    cx: Scope,
    /// The starting value for the counter
    initial_value: i32,
    /// The change that should be applied each time the button is clicked.
    step: i32,
) -> impl IntoView {
    const ESCAPE_KEY: u32 = 27;
    const ENTER_KEY: u32 = 13;

    websys_webrtc_transport::run();

    let (addr, set_addr) = create_signal::<String>(cx, "".to_owned());

    // Callback to add a todo on pressing the `Enter` key, if the field isn't empty
    let input_ref = create_node_ref::<Input>(cx);
    let handle_connect = move |ev: web_sys::KeyboardEvent| {
        let input = input_ref.get().unwrap();
        ev.stop_propagation();
        let key_code = ev.key_code();
        if key_code == ENTER_KEY {
            let multi_addr = input.value();
            let multi_addr = multi_addr.trim();
            if !multi_addr.is_empty() {
                // connecting...
                set_addr.update(|a| *a = multi_addr.to_owned());
                input.set_value("");
            }
        }
    };

    view! { cx,
        <div class="p-2">
            <div class="p-1">
                <h1>"Enter Server Multiaddr:"</h1>
                <input
                    node_ref=input_ref
                    placeholder="ip1/::/webrtc-direct/certhash"
                    autofocus
                    on:keydown=handle_connect
                    node_ref=input_ref
                    class="w-full p-2 border rounded" />
            </div>

            <Show
                when=move || addr.get().is_empty()
                fallback=move |cx| view! { cx,
                    <span>
                    "Connecting to " {addr.get().clone()}
                    </span>
                 }
            >
                "Enter Multiaddr to connect"
            </Show>
        </div>
    }
}
