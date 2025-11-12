use yew::prelude::*;
use yew::TargetCast;

use pulldown_cmark::{html as md_html, Options, Parser, Event, Tag, TagEnd};
use gloo_storage::{LocalStorage, Storage};
use web_sys::{HtmlInputElement, Element, HtmlAnchorElement};
use js_sys::Function;
use gloo_file::File;
use gloo_file::callbacks::{FileReader, read_as_text};
use std::collections::{HashMap, HashSet};
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
    let preview_expanded = use_state(|| false);
    let active_view = use_state(|| "editor".to_string());
    let close_timer = use_mut_ref(|| None::<Timeout>);

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

    let (toc, preview_html) = {
        let content = (*editor_content).clone();
        let parsed_content = use_memo(
            content,
            |content| {
                let mut options = Options::empty();
                options.insert(Options::ENABLE_HEADING_ATTRIBUTES);

                let mut toc_items = Vec::new();
                let mut current_heading = None;
                let mut html_output = String::new();
                let mut used_anchors = HashSet::new();

                let parser = Parser::new_ext(content, options);
                md_html::push_html(
                    &mut html_output,
                    parser.map(|event| {
                        match &event {
                            Event::Start(Tag::Heading { level, .. }) => {
                                current_heading = Some((*level, String::new()));
                            }
                            Event::Text(text) => {
                                if let Some((_, current_text)) = &mut current_heading {
                                    current_text.push_str(text);
                                }
                            }
                            Event::End(TagEnd::Heading(_)) => {
                                if let Some((level, text)) = current_heading.take() {
                                    let mut anchor = text
                                        .to_lowercase()
                                        .replace(' ', "-")
                                        .chars()
                                        .filter(|c| c.is_alphanumeric() || *c == '-')
                                        .collect::<String>();
                                    let mut counter = 1;
                                    let mut unique_anchor = anchor.clone();
                                    while used_anchors.contains(&unique_anchor) {
                                        unique_anchor = format!("{}-{}", anchor, counter);
                                        counter += 1;
                                    }
                                    used_anchors.insert(unique_anchor.clone());
                                    anchor = unique_anchor;

                                    let (li_class, a_class, prefix) = match level {
                                        pulldown_cmark::HeadingLevel::H1 => (
                                            "mt-3",
                                            "font-semibold text-sm text-gray-800 dark:text-gray-200",
                                            ""
                                        ),
                                        pulldown_cmark::HeadingLevel::H2 => (
                                            "mt-1",
                                            "text-sm text-gray-600 dark:text-gray-400",
                                            ""
                                        ),
                                        _ => { // H3+
                                            ("mt-1 ml-4", "text-sm text-gray-600 dark:text-gray-400", ">")
                                        }
                                    };

                                    let prefix_span = if prefix.is_empty() {
                                        "".to_string()
                                    } else {
                                        format!("<span class=\"mr-2 text-gray-400 dark:text-gray-500\">{}</span>", prefix)
                                    };

                                    toc_items.push(format!(
                                        "<li class=\"flex items-center {}\">{}{}<a href=\"#{}\" class=\"hover:text-blue-500 {}\">{}</a></li>",
                                        li_class,
                                        prefix_span,
                                        "", // no space needed
                                        anchor,
                                        a_class,
                                        text
                                    ));
                                }
                            }
                            _ => {}
                        }
                        event
                    }),
                );
                (
                    format!("<ul class=\"list-none pl-0\">{}</ul>", toc_items.join("")),
                    html_output,
                )
            },
        );
        (*parsed_content).clone()
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
                            <div class="relative"
                                onmouseenter={{
                                    let dropdown_open = dropdown_open.clone();
                                    let close_timer = close_timer.clone();
                                    Callback::from(move |_| {
                                        if let Some(timer) = close_timer.borrow_mut().take() {
                                            timer.cancel();
                                        }
                                        dropdown_open.set(true);
                                    })
                                }}
                                onmouseleave={{
                                    let dropdown_open = dropdown_open.clone();
                                    let close_timer = close_timer.clone();
                                    Callback::from(move |_| {
                                        let dropdown_open = dropdown_open.clone();
                                        let timer = Timeout::new(200, move || {
                                            dropdown_open.set(false);
                                        });
                                        *close_timer.borrow_mut() = Some(timer);
                                    })
                                }}
                            >
                                <button
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
                    "h-full", "transition-all", "duration-300",
                    if *active_view == "editor" { "block" } else { "hidden" },
                    if *preview_expanded { "md:hidden" } else { "md:block" }
                )}>
                     <textarea
                        oninput={on_input}
                        value={(*editor_content).clone()}
                        class="w-full h-full p-4 rounded-lg border border-gray-300 dark:border-gray-700 bg-white dark:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                </div>
                <div class={classes!(
                    "preview-pane", "relative", "h-full", "rounded-lg", "border", "border-gray-300", "dark:border-gray-700", "bg-white", "dark:bg-gray-800", "prose", "dark:prose-invert", "max-w-none", "transition-all", "duration-300",
                    if *preview_expanded {
                        "md:col-span-2"
                    } else {
                        "p-4 overflow-y-auto"
                    },
                    if *active_view == "preview" { "block" } else { "hidden" },
                    "md:block"
                )}>
                    <button
                        onclick={{
                            let preview_expanded = preview_expanded.clone();
                            Callback::from(move |_| {
                                preview_expanded.set(!*preview_expanded);
                            })
                        }}
                        class="absolute top-2 left-2 p-2 rounded-full bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 focus:outline-none z-10"
                    >
                        { if *preview_expanded {
                            html! {
                                <svg xmlns="http://www.w.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M15 12H9m12 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            }
                        } else {
                            html! {
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3m0 0v3m0-3h3m-3 0H9m12 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            }
                        }}
                    </button>
                    { if *preview_expanded {
                        html! {
                            <div class="grid grid-cols-12 gap-4 h-full">
                                <div class="col-span-3 border-r border-gray-200 dark:border-gray-700 h-full overflow-y-auto">
                                    <div class="toc sticky top-0 p-4">
                                        <h3 class="text-lg font-semibold mb-2">{ "On this page" }</h3>
                                        { Html::from_html_unchecked(toc.into()) }
                                    </div>
                                </div>
                                <div class="col-span-9 h-full overflow-y-auto">
                                    <div class="prose dark:prose-invert max-w-none p-4">
                                        { Html::from_html_unchecked(preview_html.into()) }
                                    </div>
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <>
                                <div class="toc sticky top-0 bg-white dark:bg-gray-800 p-4 rounded-lg border-b border-gray-300 dark:border-gray-700 mb-4 max-h-48 overflow-y-auto">
                                    <h3 class="text-lg font-semibold mb-2">{ "On this page" }</h3>
                                    { Html::from_html_unchecked(toc.into()) }
                                </div>
                                <div class="prose dark:prose-invert max-w-none">
                                    { Html::from_html_unchecked(preview_html.into()) }
                                </div>
                            </>
                        }
                    }}
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
