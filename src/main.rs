use yew::prelude::*;
use pulldown_cmark::{html, Parser};

#[function_component(App)]
fn app() -> Html {
    //
    // Holds the content of the markdown editor.
    let editor_content = use_state(|| String::from("# Rust Markdown Studio\n\nHello, world!"));

    // Callback for the textarea's oninput event.
    let on_input = {
        let editor_content = editor_content.clone();
        Callback::from(move |e: InputEvent| {
            // When the textarea content changes, update the state.
            let target = e.target_dyn_into::<web_sys::HtmlTextAreaElement>();
            if let Some(textarea) = target {
                editor_content.set(textarea.value());
            }
        })
    };

    // Parse the markdown content and render it to HTML.
    let preview_html = {
        let parser = Parser::new(&editor_content);
        let mut html_output = String::new();
        html::push_html(&mut html_output, parser);
        html_output
    };

    html! {
        <main class="container">
            <div class="editor-pane">
                <textarea oninput={on_input} value={(*editor_content).clone()} />
            </div>
            <div class="preview-pane">
                { Html::from_html_unchecked(preview_html.into()) }
            </div>
        </main>
    }
}

fn main() {
    // Mount the App component to the #app element in index.html
    yew::Renderer::<App>::new().render();
}
