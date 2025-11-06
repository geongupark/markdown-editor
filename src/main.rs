mod components;
use components::app::App;

fn main() {
    // Set the panic hook to log errors to the console
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    yew::Renderer::<App>::new().render();
}
