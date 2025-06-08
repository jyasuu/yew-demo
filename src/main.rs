mod tutorial;
mod tomato_clock;

fn main() {
    
    yew::Renderer::<tomato_clock::TomatoClockApp>::new().render();
}