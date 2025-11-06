# Gus Markdown Editor

A real-time, browser-based Markdown editor built with Rust and WebAssembly.

## 🚀 Service URL

https://geongupark.github.io/markdown-editor/

## ✨ Features

*   Live preview of rendered Markdown.
*   Syntax highlighting for code blocks.
*   Light and Dark theme support.
*   File import and export (Markdown and HTML).
*   State persisted in Local Storage.

## 🛠️ Tech Stack

*   **Framework:** [Yew](https://yew.rs/) (Rust/WASM)
*   **Build Tool:** [Trunk](https://trunkrs.dev/)
*   **Styling:** [Tailwind CSS](https://tailwindcss.com/)
*   **Markdown Parsing:** [pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)
*   **Syntax Highlighting:** [highlight.js](https://highlightjs.org/)

## 🚀 Getting Started

### Prerequisites

1.  **Rust:** Install the Rust toolchain using `rustup`.
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2.  **WASM Target:** Add the WASM target for Rust.
    ```bash
    rustup target add wasm32-unknown-unknown
    ```
3.  **Node.js:** Install Node.js and npm to manage CSS dependencies.
4.  **Trunk:** Install the Trunk build tool.
    ```bash
    cargo install trunk --locked
    ```

### Installation & Running

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/GusHeredia/gus-markdown-editor.git
    cd gus-markdown-editor
    ```
2.  **Install CSS dependencies:**
    ```bash
    npm install
    ```
3.  **Build the CSS:**
    ```bash
    npm run build-css
    ```
4.  **Serve the application:**
    ```bash
    trunk serve
    ```
5.  Open your browser and navigate to `http://127.0.0.1:8080`.
