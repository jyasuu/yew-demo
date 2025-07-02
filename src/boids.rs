use yew::prelude::*;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;
use gloo_timers::callback::Interval;
use rand::prelude::*;
use rand::thread_rng;
use rand::Rng;

#[derive(Clone, Copy, Debug)]
struct Vector2 {
    x: f64,
    y: f64,
}

impl Vector2 {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag > 0.0 {
            Self {
                x: self.x / mag,
                y: self.y / mag,
            }
        } else {
            *self
        }
    }

    fn limit(&self, max: f64) -> Self {
        if self.magnitude() > max {
            let normalized = self.normalize();
            Self {
                x: normalized.x * max,
                y: normalized.y * max,
            }
        } else {
            *self
        }
    }

    fn distance_to(&self, other: &Vector2) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl std::ops::Add for Vector2 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::Sub for Vector2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::Mul<f64> for Vector2 {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl std::ops::Div<f64> for Vector2 {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

#[derive(Clone)]
struct Boid {
    position: Vector2,
    velocity: Vector2,
    acceleration: Vector2,
    max_speed: f64,
    max_force: f64,
}

impl Boid {
    fn new(x: f64, y: f64) -> Self {
        let mut rng = thread_rng();
        Self {
            position: Vector2::new(x, y),
            velocity: Vector2::new(
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
            ),
            acceleration: Vector2::zero(),
            max_speed: 2.0,
            max_force: 0.03,
        }
    }

    fn update(&mut self, width: f64, height: f64) {
        // Update velocity
        self.velocity = self.velocity + self.acceleration;
        self.velocity = self.velocity.limit(self.max_speed);
        
        // Update position
        self.position = self.position + self.velocity;
        
        // Reset acceleration
        self.acceleration = Vector2::zero();
        
        // Wrap around edges
        if self.position.x < 0.0 {
            self.position.x = width;
        } else if self.position.x > width {
            self.position.x = 0.0;
        }
        
        if self.position.y < 0.0 {
            self.position.y = height;
        } else if self.position.y > height {
            self.position.y = 0.0;
        }
    }

    fn apply_force(&mut self, force: Vector2) {
        self.acceleration = self.acceleration + force;
    }

    fn seek(&self, target: Vector2) -> Vector2 {
        let desired = (target - self.position).normalize() * self.max_speed;
        let steer = (desired - self.velocity).limit(self.max_force);
        steer
    }

    fn separate(&self, boids: &[Boid]) -> Vector2 {
        let desired_separation = 25.0;
        let mut steer = Vector2::zero();
        let mut count = 0;

        for other in boids {
            let d = self.position.distance_to(&other.position);
            if d > 0.0 && d < desired_separation {
                let diff = (self.position - other.position).normalize() / d;
                steer = steer + diff;
                count += 1;
            }
        }

        if count > 0 {
            steer = steer / count as f64;
            steer = steer.normalize() * self.max_speed;
            steer = (steer - self.velocity).limit(self.max_force);
        }
        steer
    }

    fn align(&self, boids: &[Boid]) -> Vector2 {
        let neighbor_dist = 50.0;
        let mut sum = Vector2::zero();
        let mut count = 0;

        for other in boids {
            let d = self.position.distance_to(&other.position);
            if d > 0.0 && d < neighbor_dist {
                sum = sum + other.velocity;
                count += 1;
            }
        }

        if count > 0 {
            sum = sum / count as f64;
            sum = sum.normalize() * self.max_speed;
            let steer = (sum - self.velocity).limit(self.max_force);
            steer
        } else {
            Vector2::zero()
        }
    }

    fn cohesion(&self, boids: &[Boid]) -> Vector2 {
        let neighbor_dist = 50.0;
        let mut sum = Vector2::zero();
        let mut count = 0;

        for other in boids {
            let d = self.position.distance_to(&other.position);
            if d > 0.0 && d < neighbor_dist {
                sum = sum + other.position;
                count += 1;
            }
        }

        if count > 0 {
            sum = sum / count as f64;
            self.seek(sum)
        } else {
            Vector2::zero()
        }
    }

    fn flock(&mut self, boids: &[Boid], sep_weight: f64, ali_weight: f64, coh_weight: f64) {
        let sep = self.separate(boids) * sep_weight;
        let ali = self.align(boids) * ali_weight;
        let coh = self.cohesion(boids) * coh_weight;

        self.apply_force(sep);
        self.apply_force(ali);
        self.apply_force(coh);
    }

    fn draw(&self, ctx: &CanvasRenderingContext2d) {
        let angle = self.velocity.y.atan2(self.velocity.x);
        
        ctx.save();
        ctx.translate(self.position.x, self.position.y).unwrap();
        ctx.rotate(angle).unwrap();
        
        ctx.begin_path();
        ctx.move_to(8.0, 0.0);
        ctx.line_to(-8.0, -3.0);
        ctx.line_to(-8.0, 3.0);
        ctx.close_path();
        
        ctx.set_fill_style(&"#4CAF50".into());
        ctx.fill();
        ctx.set_stroke_style(&"#2E7D32".into());
        ctx.set_line_width(1.0);
        ctx.stroke();
        
        ctx.restore();
    }
}

#[derive(Properties, PartialEq)]
pub struct BoidsProps {}

pub struct BoidsApp {
    canvas_ref: NodeRef,
    boids: Rc<RefCell<Vec<Boid>>>,
    _interval: Option<Interval>,
    separation_weight: f64,
    alignment_weight: f64,
    cohesion_weight: f64,
    num_boids: usize,
}

pub enum Msg {
    Tick,
    UpdateSeparation(f64),
    UpdateAlignment(f64),
    UpdateCohesion(f64),
    UpdateNumBoids(usize),
    AddBoid(f64, f64),
}

impl Component for BoidsApp {
    type Message = Msg;
    type Properties = BoidsProps;

    fn create(ctx: &Context<Self>) -> Self {
        let boids = Rc::new(RefCell::new(Vec::new()));
        
        // Initialize with some boids
        {
            let mut boids_ref = boids.borrow_mut();
            let mut rng = thread_rng();
            for _ in 0..50 {
                boids_ref.push(Boid::new(
                    rng.gen_range(0.0..800.0),
                    rng.gen_range(0.0..600.0),
                ));
            }
        }

        let link = ctx.link().clone();
        let interval = Interval::new(16, move || {
            link.send_message(Msg::Tick);
        });

        Self {
            canvas_ref: NodeRef::default(),
            boids,
            _interval: Some(interval),
            separation_weight: 1.5,
            alignment_weight: 1.0,
            cohesion_weight: 1.0,
            num_boids: 50,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                self.animate();
                false
            }
            Msg::UpdateSeparation(weight) => {
                self.separation_weight = weight;
                false
            }
            Msg::UpdateAlignment(weight) => {
                self.alignment_weight = weight;
                false
            }
            Msg::UpdateCohesion(weight) => {
                self.cohesion_weight = weight;
                false
            }
            Msg::UpdateNumBoids(num) => {
                self.num_boids = num;
                self.update_boid_count();
                false
            }
            Msg::AddBoid(x, y) => {
                self.boids.borrow_mut().push(Boid::new(x, y));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_canvas_click = {
            let link = ctx.link().clone();
            Callback::from(move |e: MouseEvent| {
                let x = e.offset_x() as f64;
                let y = e.offset_y() as f64;
                link.send_message(Msg::AddBoid(x, y));
            })
        };

        let on_separation_change = {
            let link = ctx.link().clone();
            Callback::from(move |e: Event| {
                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                let value = input.value().parse::<f64>().unwrap_or(1.0);
                link.send_message(Msg::UpdateSeparation(value));
            })
        };

        let on_alignment_change = {
            let link = ctx.link().clone();
            Callback::from(move |e: Event| {
                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                let value = input.value().parse::<f64>().unwrap_or(1.0);
                link.send_message(Msg::UpdateAlignment(value));
            })
        };

        let on_cohesion_change = {
            let link = ctx.link().clone();
            Callback::from(move |e: Event| {
                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                let value = input.value().parse::<f64>().unwrap_or(1.0);
                link.send_message(Msg::UpdateCohesion(value));
            })
        };

        let on_num_boids_change = {
            let link = ctx.link().clone();
            Callback::from(move |e: Event| {
                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                let value = input.value().parse::<usize>().unwrap_or(50);
                link.send_message(Msg::UpdateNumBoids(value));
            })
        };

        html! {
            <div style="padding: 20px; font-family: Arial, sans-serif;">
                <h1>{"Interactive Boids Algorithm Demo"}</h1>
                
                <div style="margin-bottom: 20px;">
                    <h3>{"Controls"}</h3>
                    <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 15px; margin-bottom: 15px;">
                        <div>
                            <label>{"Separation: "}{self.separation_weight}</label>
                            <input 
                                type="range" 
                                min="0" 
                                max="3" 
                                step="0.1" 
                                value={self.separation_weight.to_string()}
                                onchange={on_separation_change}
                                style="width: 100%;"
                            />
                        </div>
                        <div>
                            <label>{"Alignment: "}{self.alignment_weight}</label>
                            <input 
                                type="range" 
                                min="0" 
                                max="3" 
                                step="0.1" 
                                value={self.alignment_weight.to_string()}
                                onchange={on_alignment_change}
                                style="width: 100%;"
                            />
                        </div>
                        <div>
                            <label>{"Cohesion: "}{self.cohesion_weight}</label>
                            <input 
                                type="range" 
                                min="0" 
                                max="3" 
                                step="0.1" 
                                value={self.cohesion_weight.to_string()}
                                onchange={on_cohesion_change}
                                style="width: 100%;"
                            />
                        </div>
                        <div>
                            <label>{"Number of Boids: "}{self.num_boids}</label>
                            <input 
                                type="range" 
                                min="10" 
                                max="200" 
                                step="10" 
                                value={self.num_boids.to_string()}
                                onchange={on_num_boids_change}
                                style="width: 100%;"
                            />
                        </div>
                    </div>
                    <p style="color: #666; font-size: 14px;">
                        {"Click on the canvas to add more boids!"}
                    </p>
                </div>

                <canvas
                    ref={self.canvas_ref.clone()}
                    width="800"
                    height="600"
                    onclick={on_canvas_click}
                    style="border: 2px solid #333; background: #f0f8ff; cursor: crosshair;"
                />
                
                <div style="margin-top: 20px;">
                    <h3>{"About the Boids Algorithm"}</h3>
                    <p>{"This demo implements Craig Reynolds' boids algorithm with three basic rules:"}</p>
                    <ul>
                        <li><strong>{"Separation"}</strong>{": Avoid crowding neighbors"}</li>
                        <li><strong>{"Alignment"}</strong>{": Steer towards average heading of neighbors"}</li>
                        <li><strong>{"Cohesion"}</strong>{": Steer towards average position of neighbors"}</li>
                    </ul>
                    <p>{"Adjust the sliders to see how each rule affects the flocking behavior!"}</p>
                </div>
            </div>
        }
    }
}

impl BoidsApp {
    fn animate(&self) {
        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>();
        if let Some(canvas) = canvas {
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
                .unwrap();
            
            let width = canvas.width() as f64;
            let height = canvas.height() as f64;
            
            // Clear canvas
            ctx.clear_rect(0.0, 0.0, width, height);
            
            // Update and draw boids
            {
                let mut boids = self.boids.borrow_mut();
                let boids_copy = boids.clone();
                
                for boid in boids.iter_mut() {
                    boid.flock(&boids_copy, self.separation_weight, self.alignment_weight, self.cohesion_weight);
                    boid.update(width, height);
                    boid.draw(&ctx);
                }
            }
        }
    }

    fn update_boid_count(&self) {
        let mut boids = self.boids.borrow_mut();
        let current_count = boids.len();
        
        if current_count < self.num_boids {
            // Add boids
            let mut rng = thread_rng();
            for _ in current_count..self.num_boids {
                boids.push(Boid::new(
                    rng.gen_range(0.0..800.0),
                    rng.gen_range(0.0..600.0),
                ));
            }
        } else if current_count > self.num_boids {
            // Remove boids
            boids.truncate(self.num_boids);
        }
    }
}

