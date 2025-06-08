mod tutorial;
mod tomato_clock;
mod timer;

fn main() {
    // yew::Renderer::<timer::App>::new().render();
    yew::Renderer::<tomato_clock::TomatoClockApp>::new().render();
}