use yew::prelude::*;
use yew::TargetCast;

use pulldown_cmark::{html as md_html, Parser};
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlInputElement, Element, HtmlAnchorElement};
use js_sys::Function;
use gloo_file::File;
use gloo_file::callbacks::{FileReader, read_as_text};
use std::collections::HashMap;
use wasm_bindgen::{prelude::*, JsCast};
use yew_hooks::prelude::*;
use gloo_timers::callback::Timeout;

const MARKDOWN_KEY: &str = "gus-markdown-editor-content";
const THEME_KEY: &str = "gus-markdown-editor-theme";

#[function_component(App)]
pub fn app() -> Html {
    let editor_content = use_state(|| {
        LocalStorage::get(MARKDOWN_KEY)
            .unwrap_or_else(|_| {
                [
                    "# Gus Markdown Editor",
                    "## Features",
                    "### Code Highlighting",
                    "```rust",
                    "fn main() {",
                    "    println!(\"Hello, world!\");",
                    "}",
                    "```",
                ].join("\n")
            })
    });

    let theme = use_state(|| {
        LocalStorage::get(THEME_KEY).unwrap_or_else(|_| "light".to_string())
    });
    let dropdown_open = use_state(|| false);
    let active_view = use_state(|| "editor".to_string());

    {
        let theme = theme.clone();
        use_effect_with(theme.clone(), move |_| {
            let document = web_sys::window().unwrap().document().unwrap();
            let html = document.document_element().unwrap().dyn_into::<Element>().unwrap();
            if *theme == "dark" {
                html.class_list().add_1("dark").unwrap();
            } else {
                html.class_list().remove_1("dark").unwrap();
            }
            LocalStorage::set(THEME_KEY, &*theme).expect("Failed to save theme to LocalStorage");
        });
    }

    let on_toggle_theme = {
        let theme = theme.clone();
        Callback::from(move |_| {
            if *theme == "light" {
                theme.set("dark".to_string());
            } else {
                theme.set("light".to_string());
            }
        })
    };

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

    {
        let preview_html = preview_html.clone();
        use_effect_with(preview_html, move |_| {
            let timeout = Timeout::new(1, move || {
                let window = web_sys::window().unwrap();
                if let Ok(hljs) = js_sys::Reflect::get(&window, &"hljs".into()) {
                    if let Ok(highlight_all) = js_sys::Reflect::get(&hljs, &"highlightAll".into()) {
                        if let Some(highlight_all_fn) = highlight_all.dyn_ref::<Function>() {
                            highlight_all_fn.call0(&wasm_bindgen::JsValue::undefined()).unwrap();
                        }
                    }
                }
            });
            timeout.forget();
        });
    }

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
            let a = web_sys::window().unwrap().document().unwrap().create_element("a").unwrap().dyn_into::<HtmlAnchorElement>().unwrap();
            a.set_href(&url);
            a.set_download("gus-markdown-editor.md");
            a.click();
            web_sys::Url::revoke_object_url(&url).unwrap();
        })
    };

    let on_export_html = {
        let preview_html = preview_html.clone();
        Callback::from(move |_| {
            let blob = web_sys::Blob::new_with_str_sequence(&js_sys::Array::of1(&JsValue::from_str(&preview_html))).unwrap();
            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
            let a = web_sys::window().unwrap().document().unwrap().create_element("a").unwrap().dyn_into::<HtmlAnchorElement>().unwrap();
            a.set_href(&url);
            a.set_download("gus-markdown-editor.html");
            a.click();
            web_sys::Url::revoke_object_url(&url).unwrap();
        })
    };

    html! {
        <div class="flex flex-col min-h-screen bg-gray-100 dark:bg-gray-900 text-gray-800 dark:text-gray-200">
            <header class="bg-white dark:bg-gray-800 shadow-md sticky top-0 z-10">
                <nav class="container mx-auto px-4 sm:px-6 py-3">
                    <div class="flex flex-wrap items-center justify-between">
                        <div class="text-xl font-semibold">
                            { "Gus Markdown Editor" }
                        </div>
                        <div class="flex items-center space-x-2 sm:space-x-4">
                            <div class="relative">
                                <button
                                    onclick={{
                                        let dropdown_open = dropdown_open.clone();
                                        Callback::from(move |_| dropdown_open.set(!*dropdown_open))
                                    }}
                                    class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-200 dark:hover:bg-gray-700 focus:outline-none"
                                >
                                    { "File" }
                                </button>
                                { if *dropdown_open {
                                    html! {
                                        <div class="absolute right-0 mt-2 w-48 bg-white dark:bg-gray-800 rounded-md shadow-lg py-1 z-20">
                                            <button onclick={on_import_md.clone()} class="block w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700">{ "Import .md" }</button>
                                            <button onclick={on_export_md.clone()} class="block w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700">{ "Export .md" }</button>
                                            <button onclick={on_export_html.clone()} class="block w-full text-left px-4 py-2 text-sm text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700">{ "Export .html" }</button>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                                 <a href="https://github.com/geongupark/markdown-editor/blob/main/README.md" target="_blank" class="px-3 py-2 rounded-md text-sm font-medium hover:bg-gray-200 dark:hover:bg-gray-700 focus:outline-none">
                                { "Doc" }
                            </a>
                            <button onclick={on_toggle_theme} class="p-2 rounded-full hover:bg-gray-200 dark:hover:bg-gray-700 focus:outline-none">
                                { if *theme == "light" {
                                    html! { <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" /></svg> }
                                } else {
                                    html! { <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" /></svg> }
                                }}
                            </button>
                        </div>
                    </div>
                </nav>
            </header>

            <div class="md:hidden">
                <div class="flex border-b border-gray-200 dark:border-gray-700">
                    <button
                        onclick={{
                            let active_view = active_view.clone();
                            Callback::from(move |_| active_view.set("editor".to_string()))
                        }}
                        class={classes!(
                            "flex-1", "py-2", "px-4", "text-center",
                            if *active_view == "editor" { "bg-gray-200 dark:bg-gray-700" } else { "" }
                        )}
                    >
                        { "Editor" }
                    </button>
                    <button
                        onclick={{
                            let active_view = active_view.clone();
                            Callback::from(move |_| active_view.set("preview".to_string()))
                        }}
                        class={classes!(
                            "flex-1", "py-2", "px-4", "text-center",
                            if *active_view == "preview" { "bg-gray-200 dark:bg-gray-700" } else { "" }
                        )}
                    >
                        { "Preview" }
                    </button>
                </div>
            </div>

            <main ref={node} class="flex-grow container mx-auto p-4 flex flex-col md:grid md:grid-cols-2 md:gap-4 h-full">
                <div class={classes!(
                    "h-full",
                    if *active_view == "editor" { "block" } else { "hidden" },
                    "md:block"
                )}>
                     <textarea
                        oninput={on_input}
                        value={(*editor_content).clone()}
                        class="w-full h-full p-4 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                </div>
                <div class={classes!(
                    "preview-pane", "h-full", "p-4", "rounded-lg", "border", "border-gray-300", "dark:border-gray-700", "bg-white", "dark:bg-gray-800", "overflow-y-auto", "prose", "dark:prose-invert", "max-w-none",
                    if *active_view == "preview" { "block" } else { "hidden" },
                    "md:block"
                )}>
                    { Html::from_html_unchecked(preview_html.into()) }
                </div>
            </main>

            <footer class="bg-white dark:bg-gray-800 py-4 mt-auto">
                <div class="container mx-auto px-6 text-center text-sm">
                    <p>{ "© 2024 Gus Markdown Editor. All rights reserved." }</p>
                </div>
            </footer>
        </div>
    }
}
