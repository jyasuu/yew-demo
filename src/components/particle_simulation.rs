#![allow(dead_code)]
use yew::prelude::*;
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;
use rand::Rng;

#[derive(Clone, Debug)]
struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    radius: f64,
    mass: f64,
    color: String,
    life: f64,
    max_life: f64,
}

#[warn(unused_unsafe)]
impl Particle {
    fn new(x: f64, y: f64) -> Self {
        let angle = rand::rng().random::<f64>() * 2.0 * std::f64::consts::PI;
        let speed =  rand::rng().random::<f64>() * 5.0 + 2.0;
        
        Self {
            x,
            y,
            vx: angle.cos() * speed,
            vy: angle.sin() * speed,
            radius: rand::rng().random::<f64>() * 3.0 + 2.0,
            mass: rand::rng().random::<f64>() * 2.0 + 1.0,
            color: format!(
                "hsl({}, 70%, 60%)",
                (rand::rng().random::<f64>() * 360.0) as i32
            ),
            life: 255.0,
            max_life: 255.0,
        }
    }
    
    fn update(&mut self, width: f64, height: f64, gravity: f64, damping: f64) {
        // Apply gravity
        self.vy += gravity;
        
        // Update position
        self.x += self.vx;
        self.y += self.vy;
        
        // Boundary collisions with energy loss
        if self.x - self.radius <= 0.0 || self.x + self.radius >= width {
            self.vx *= -damping;
            self.x = self.x.max(self.radius).min(width - self.radius);
        }
        
        if self.y - self.radius <= 0.0 || self.y + self.radius >= height {
            self.vy *= -damping;
            self.y = self.y.max(self.radius).min(height - self.radius);
        }
        
        // Decrease life
        self.life -= 1.0;
    }
    
    fn is_alive(&self) -> bool {
        self.life > 0.0
    }
    
    fn draw(&self, ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        let alpha = self.life / self.max_life;
        ctx.set_global_alpha(alpha);
        
        ctx.begin_path();
        ctx.arc(self.x, self.y, self.radius, 0.0, 2.0 * std::f64::consts::PI)?;
        ctx.set_fill_style_str(&self.color);
        ctx.fill();
        
        // Add glow effect
        ctx.set_shadow_blur(10.0);
        ctx.set_shadow_color(&self.color);
        ctx.fill();
        
        ctx.set_shadow_blur(0.0);
        ctx.set_global_alpha(1.0);
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct PhysicsSettings {
    gravity: f64,
    damping: f64,
    max_particles: usize,
    particle_spawn_rate: f64,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: 0.2,
            damping: 0.8,
            max_particles: 200,
            particle_spawn_rate: 0.3,
        }
    }
}

pub struct ParticleSimulation {
    particles: Rc<RefCell<Vec<Particle>>>,
    canvas_ref: NodeRef,
    animation_id: Rc<RefCell<Option<i32>>>,
    settings: PhysicsSettings,
    mouse_x: f64,
    mouse_y: f64,
    is_mouse_down: bool,
}

pub enum Msg {
    StartAnimation,
    StopAnimation,
    MouseMove(MouseEvent),
    MouseDown(MouseEvent),
    MouseUp,
    UpdateGravity(String),
    UpdateDamping(String),
    UpdateMaxParticles(String),
    ClearParticles,
}

impl Component for ParticleSimulation {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            particles: Rc::new(RefCell::new(Vec::new())),
            canvas_ref: NodeRef::default(),
            animation_id: Rc::new(RefCell::new(None)),
            settings: PhysicsSettings::default(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            is_mouse_down: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::StartAnimation => {
                self.start_animation(ctx.link().clone());
                false
            }
            Msg::StopAnimation => {
                self.stop_animation();
                false
            }
            Msg::MouseMove(event) => {
                self.mouse_x = event.offset_x() as f64;
                self.mouse_y = event.offset_y() as f64;
                false
            }
            Msg::MouseDown(event) => {
                self.mouse_x = event.offset_x() as f64;
                self.mouse_y = event.offset_y() as f64;
                self.is_mouse_down = true;
                false
            }
            Msg::MouseUp => {
                self.is_mouse_down = false;
                false
            }
            Msg::UpdateGravity(value) => {
                if let Ok(gravity) = value.parse::<f64>() {
                    self.settings.gravity = gravity;
                }
                false
            }
            Msg::UpdateDamping(value) => {
                if let Ok(damping) = value.parse::<f64>() {
                    self.settings.damping = damping.max(0.0).min(1.0);
                }
                false
            }
            Msg::UpdateMaxParticles(value) => {
                if let Ok(max) = value.parse::<usize>() {
                    self.settings.max_particles = max.min(1000);
                }
                false
            }
            Msg::ClearParticles => {
                self.particles.borrow_mut().clear();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        
        html! {
            <div class="particle-simulation">
                <div class="controls">
                    <h2>{ "Particle Physics Simulation" }</h2>
                    
                    <div class="control-group">
                        <button onclick={link.callback(|_| Msg::StartAnimation)}>
                            { "Start" }
                        </button>
                        <button onclick={link.callback(|_| Msg::StopAnimation)}>
                            { "Stop" }
                        </button>
                        <button onclick={link.callback(|_| Msg::ClearParticles)}>
                            { "Clear" }
                        </button>
                    </div>
                    
                    <div class="physics-controls">
                        <div class="control">
                            <label>{ "Gravity: " }</label>
                            <input 
                                type="range" 
                                min="0" 
                                max="1" 
                                step="0.01"
                                value={self.settings.gravity.to_string()}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::UpdateGravity(input.value())
                                })}
                            />
                            <span>{ format!("{:.2}", self.settings.gravity) }</span>
                        </div>
                        
                        <div class="control">
                            <label>{ "Damping: " }</label>
                            <input 
                                type="range" 
                                min="0" 
                                max="1" 
                                step="0.01"
                                value={self.settings.damping.to_string()}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::UpdateDamping(input.value())
                                })}
                            />
                            <span>{ format!("{:.2}", self.settings.damping) }</span>
                        </div>
                        
                        <div class="control">
                            <label>{ "Max Particles: " }</label>
                            <input 
                                type="range" 
                                min="50" 
                                max="500" 
                                step="10"
                                value={self.settings.max_particles.to_string()}
                                oninput={link.callback(|e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    Msg::UpdateMaxParticles(input.value())
                                })}
                            />
                            <span>{ self.settings.max_particles }</span>
                        </div>
                    </div>
                    
                    <p class="instructions">
                        { "Click and drag on the canvas to spawn particles!" }
                    </p>
                </div>
                
                <canvas 
                    ref={self.canvas_ref.clone()}
                    width="800" 
                    height="600"
                    onmousemove={link.callback(Msg::MouseMove)}
                    onmousedown={link.callback(Msg::MouseDown)}
                    onmouseup={link.callback(|_| Msg::MouseUp)}
                />
                
                <style>
                {r#"
                    .particle-simulation {
                        display: flex;
                        flex-direction: column;
                        align-items: center;
                        padding: 20px;
                        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                        background: linear-gradient(135deg, #1e3c72, #2a5298);
                        min-height: 100vh;
                        color: white;
                    }
                    
                    .controls {
                        background: rgba(255, 255, 255, 0.1);
                        padding: 20px;
                        border-radius: 10px;
                        margin-bottom: 20px;
                        backdrop-filter: blur(10px);
                        border: 1px solid rgba(255, 255, 255, 0.2);
                    }
                    
                    .control-group {
                        display: flex;
                        gap: 10px;
                        margin: 15px 0;
                    }
                    
                    .physics-controls {
                        display: grid;
                        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
                        gap: 15px;
                        margin: 15px 0;
                    }
                    
                    .control {
                        display: flex;
                        align-items: center;
                        gap: 10px;
                    }
                    
                    .control label {
                        min-width: 80px;
                        font-size: 14px;
                    }
                    
                    .control input[type="range"] {
                        flex: 1;
                        min-width: 100px;
                    }
                    
                    .control span {
                        min-width: 40px;
                        font-size: 14px;
                        font-weight: bold;
                    }
                    
                    button {
                        background: linear-gradient(45deg, #667eea, #764ba2);
                        color: white;
                        border: none;
                        padding: 10px 20px;
                        border-radius: 5px;
                        cursor: pointer;
                        font-size: 14px;
                        transition: all 0.3s ease;
                    }
                    
                    button:hover {
                        transform: translateY(-2px);
                        box-shadow: 0 5px 15px rgba(0, 0, 0, 0.3);
                    }
                    
                    canvas {
                        border: 2px solid rgba(255, 255, 255, 0.3);
                        border-radius: 10px;
                        background: radial-gradient(circle at 50% 50%, rgba(0, 0, 0, 0.8), rgba(0, 0, 0, 0.9));
                        cursor: crosshair;
                        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
                    }
                    
                    .instructions {
                        text-align: center;
                        margin-top: 10px;
                        font-style: italic;
                        opacity: 0.8;
                    }
                    
                    h2 {
                        text-align: center;
                        margin: 0 0 20px 0;
                        background: linear-gradient(45deg, #667eea, #764ba2);
                        -webkit-background-clip: text;
                        -webkit-text-fill-color: transparent;
                        background-clip: text;
                    }
                "#}
                </style>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if _first_render {
            self.start_animation(ctx.link().clone());
        }
    }
}

impl ParticleSimulation {
    fn start_animation(&self, link: yew::html::Scope<Self>) {
        if self.animation_id.borrow().is_some() {
            return;
        }

        let canvas = self.canvas_ref.cast::<HtmlCanvasElement>().unwrap();
        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let width = canvas.width() as f64;
        let height = canvas.height() as f64;
        
        let particles = self.particles.clone();
        let animation_id = self.animation_id.clone();
        let settings = self.settings.clone();
        let is_mouse_down = self.is_mouse_down;
        let mouse_x = self.mouse_x;
        let mouse_y = self.mouse_y;

        let animate: Rc<RefCell<Option<Closure<dyn FnMut() + 'static>>>> = Rc::new(RefCell::new(None));
        let animate_clone = animate.clone();

        *animate.borrow_mut() = Some(Closure::wrap(Box::new(move || {
            // Clear canvas with trail effect
            ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.1)"));
            ctx.fill_rect(0.0, 0.0, width, height);

            let mut particles_vec = particles.borrow_mut();
            
            // Spawn new particles when mouse is down
            if is_mouse_down && particles_vec.len() < settings.max_particles {
                for _ in 0..3 {
                    particles_vec.push(Particle::new(mouse_x, mouse_y));
                }
            }
            
            // Randomly spawn particles
            if rand::rng().random::<f64>() < settings.particle_spawn_rate && particles_vec.len() < settings.max_particles {
                particles_vec.push(Particle::new(
                    rand::rng().random::<f64>() * width,
                    rand::rng().random::<f64>() * height * 0.3,
                ));
            }

            // Update and draw particles
            particles_vec.retain_mut(|particle| {
                particle.update(width, height, settings.gravity, settings.damping);
                
                if particle.is_alive() {
                    let _ = particle.draw(&ctx);
                    true
                } else {
                    false
                }
            });

            // Particle interactions (simple collision detection)
            let particles_clone = particles_vec.clone();
            for i in 0..particles_vec.len() {
                for j in (i + 1)..particles_clone.len() {
                    let dx = particles_vec[i].x - particles_clone[j].x;
                    let dy = particles_vec[i].y - particles_clone[j].y;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    if distance < particles_vec[i].radius + particles_clone[j].radius {
                        // Simple elastic collision
                        let angle = dy.atan2(dx);
                        let sin = angle.sin();
                        let cos = angle.cos();
                        
                        // Separate particles
                        let overlap = particles_vec[i].radius + particles_clone[j].radius - distance;
                        particles_vec[i].x += cos * overlap * 0.5;
                        particles_vec[i].y += sin * overlap * 0.5;
                        
                        // Exchange velocities (simplified)
                        let temp_vx = particles_vec[i].vx;
                        let temp_vy = particles_vec[i].vy;
                        particles_vec[i].vx = particles_clone[j].vx * 0.8;
                        particles_vec[i].vy = particles_clone[j].vy * 0.8;
                        
                        if j < particles_vec.len() {
                            particles_vec[j].vx = temp_vx * 0.8;
                            particles_vec[j].vy = temp_vy * 0.8;
                        }
                    }
                }
            }

            // Continue animation
            if let Some(window) = window() {
                let id = window.request_animation_frame(
                    animate_clone.borrow().as_ref().unwrap().as_ref().unchecked_ref()
                ).unwrap();
                *animation_id.borrow_mut() = Some(id);
            }
        }) as Box<dyn FnMut()>));

        // Start the animation
        if let Some(window) = window() {
            let id = window.request_animation_frame(
                animate.borrow().as_ref().unwrap().as_ref().unchecked_ref()
            ).unwrap();
            *self.animation_id.borrow_mut() = Some(id);
        }
    }

    fn stop_animation(&self) {
        if let Some(id) = self.animation_id.borrow_mut().take() {
            if let Some(window) = window() {
                window.cancel_animation_frame(id).ok();
            }
        }
    }
}
