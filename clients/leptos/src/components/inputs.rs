use leptos::{ev::MouseEvent, *};

#[component]
pub fn ControlledComponent(cx: Scope) -> impl IntoView {
    // create a signal to hold the value
    let (name, set_name) = create_signal(cx, "Controlled".to_string());

    view! { cx,
        <input type="text"
            // fire an event whenever the input changes
            on:input=move |ev| {
                // event_target_value is a Leptos helper function
                // it functions the same way as event.target.value
                // in JavaScript, but smooths out some of the typecasting
                // necessary to make this work in Rust
                set_name(event_target_value(&ev));
            }

            // the `prop:` syntax lets you update a DOM property,
            // rather than an attribute.
            //
            // IMPORTANT: the `value` *attribute* only sets the
            // initial value, until you have made a change.
            // The `value` *property* sets the current value.
            // This is a quirk of the DOM; I didn't invent it.
            // Other frameworks gloss this over; I think it's
            // more important to give you access to the browser
            // as it really works.
            //
            // tl;dr: use prop:value for form inputs
            prop:value=name
        />
        <p>"Name is: " {name}</p>
    }
}

#[component]
pub fn UncontrolledComponent(cx: Scope) -> impl IntoView {
    // import the type for <input>
    use leptos::html::Input;

    let (name, set_name) = create_signal(cx, "/ip6/2607:fea8:fec0:7337::d93a8:fec0:7337::d93a/udp/42069/webrtc/certhash/uEiBuHZVGRtZSmas1fc8dDmbSXUalQD7wzmBNR5XcwRb0cQ".to_string());

    // we'll use a NodeRef to store a reference to the input element
    // this will be filled when the element is created
    let input_element: NodeRef<Input> = create_node_ref(cx);

    // fires when the form `submit` event happens
    // this will store the value of the <input> in our signal
    let on_submit = move |ev: MouseEvent| {
        // stop the page from reloading!
        ev.prevent_default();

        // here, we'll extract the value from the input
        let value = input_element()
            // event handlers can only fire after the view
            // is mounted to the DOM, so the `NodeRef` will be `Some`
            .expect("<input> to exist")
            // `NodeRef` implements `Deref` for the DOM element type
            // this means we can call`HtmlInputElement::value()`
            // to get the current value of the input
            .value();
        set_name(value);
    };

    view! { cx,
        <div class="flex">
            <input type="text" class="flex-1 border p-2 mx-2 rounded"
                // here, we use the `value` *attribute* to set only
                // the initial value, letting the browser maintain
                // the state after that
                value=name

                // store a reference to this input in `input_element`
                node_ref=input_element
            />
            <button on:click=on_submit class="flex-initial border bg-green-500 rounded outline-none text-white p-2 mx-2">
                "Go!"
            </button>
        </div>
        <div class="break-words">
            <p>"Name is: " {name}</p>
        </div>
    }
}
