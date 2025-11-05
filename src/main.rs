use yew::prelude::*;
use yew::TargetCast;
use pulldown_cmark::{html as md_html, Parser};
use gloo_storage::{LocalStorage, Storage};
use web_sys::HtmlInputElement;
use gloo_file::File;
use gloo_file::callbacks::{FileReader, read_as_text};
use std::collections::HashMap;
use wasm_bindgen::{prelude::*, JsCast};
use yew_hooks::prelude::*;

const MARKDOWN_KEY: &str = "rust-markdown-studio-content";

#[function_component(App)]
fn app() -> Html {
    let editor_content = use_state(|| {
        LocalStorage::get(MARKDOWN_KEY)
            .unwrap_or_else(|_| "# Rust Markdown Studio\n\nHello, world!".to_string())
    });

    let tasks = use_mut_ref(HashMap::<String, FileReader>::new);
    let node = use_node_ref();
    let drop_state = use_drop(node.clone());

    {
        let editor_content = editor_content.clone();
        let tasks = tasks.clone();
        use_effect_with(drop_state.files.clone(), move |files| {
            if let Some(files) = &**files {
                if let Some(file) = files.get(0) {
                    let file = File::from(file.clone());
                    let file_name = file.name();
                    let editor_content = editor_content.clone();
                    let tasks = tasks.clone();

                    let file_name_clone = file_name.clone();
                    let tasks_for_closure = tasks.clone();
                    let task = read_as_text(&file, move |res| {
                        if let Ok(content) = res {
                            editor_content.set(content);
                        }
                        tasks_for_closure.borrow_mut().remove(&file_name_clone);
                    });
                    tasks.borrow_mut().insert(file_name, task);
                }
            }
        });
    }


    let on_input = {
        let editor_content = editor_content.clone();
        Callback::from(move |e: InputEvent| {
            let target = e.target_dyn_into::<web_sys::HtmlTextAreaElement>();
            if let Some(textarea) = target {
                editor_content.set(textarea.value());
            }
        })
    };

    {
        let editor_content = editor_content.clone();
        use_effect_with(editor_content.clone(), move |_| {
            LocalStorage::set(MARKDOWN_KEY, &*editor_content).expect("Failed to save to LocalStorage");
        });
    }

    let preview_html = {
        let parser = Parser::new(&editor_content);
        let mut html_output = String::new();
        md_html::push_html(&mut html_output, parser);
        html_output
    };

    let on_import_md = {
        let editor_content = editor_content.clone();
        let tasks = tasks.clone();
        Callback::from(move |_| {
            let file_input = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .create_element("input")
                .unwrap()
                .dyn_into::<HtmlInputElement>()
                .unwrap();
            file_input.set_type("file");
            file_input.set_accept(".md");

            let editor_content_clone = editor_content.clone();
            let tasks_clone = tasks.clone();
            let onchange = Closure::wrap(Box::new(move |e: yew::Event| {
                let input: HtmlInputElement = e.target_dyn_into().unwrap();
                if let Some(files) = input.files() {
                    if let Some(file) = files.get(0) {
                        let file = File::from(file);
                        let file_name = file.name();
                        let editor_content = editor_content_clone.clone();
                        let tasks = tasks_clone.clone();

                        let tasks_for_closure = tasks.clone();
                        let task = read_as_text(&file, move |res| {
                            if let Ok(content) = res {
                                editor_content.set(content);
                            }
                            tasks_for_closure.borrow_mut().remove(&file_name);
                        });
                        tasks.borrow_mut().insert(file.name(), task);
                    }
                }
            }) as Box<dyn FnMut(_)>);

            file_input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
            file_input.click();
            onchange.forget();
        })
    };

    let on_export_md = {
        let editor_content = editor_content.clone();
        Callback::from(move |_| {
            let blob = web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&JsValue::from_str(&editor_content))).unwrap();
            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
            let a = web_sys::window().unwrap().document().unwrap().create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
            a.set_href(&url);
            a.set_download("rust-markdown-studio.md");
            a.click();
            web_sys::Url::revoke_object_url(&url).unwrap();
        })
    };

    let on_export_html = {
        let preview_html = preview_html.clone();
        Callback::from(move |_| {
            let blob = web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&JsValue::from_str(&preview_html))).unwrap();
            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
            let a = web_sys::window().unwrap().document().unwrap().create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
            a.set_href(&url);
            a.set_download("rust-markdown-studio.html");
            a.click();
            web_sys::Url::revoke_object_url(&url).unwrap();
        })
    };

    html! {
        <main class="app-wrapper">
            <div class="toolbar">
                <button onclick={on_import_md}>{ "Import .md" }</button>
                <button onclick={on_export_md}>{ "Export .md" }</button>
                <button onclick={on_export_html}>{ "Export .html" }</button>
            </div>
            <div class="content-wrapper">
                <div ref={node} class="editor-pane">
                    <textarea oninput={on_input} value={(*editor_content).clone()} />
                </div>
                <div class="preview-pane">
                    { Html::from_html_unchecked(preview_html.into()) }
                </div>
            </div>
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
