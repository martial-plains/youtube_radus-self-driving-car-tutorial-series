use app::App;

pub mod app;
pub mod car;
pub mod controls;
pub mod network;
pub mod road;
pub mod sensor;
pub mod utils;
pub mod visualizer;

fn main() {
    yew::Renderer::<App>::new().render();
}
