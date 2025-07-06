use yew::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, MouseEvent};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::f64::consts::PI;
use gloo_timers::callback::Interval;
use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub enum ParticleType {
    Circle,
    Spark,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffectType {
    Fire,
    Explosion,
    Rain,
    Snow,
    Magic,
    Energy,
}

impl std::fmt::Display for EffectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectType::Fire => write!(f, "🔥 Fire"),
            EffectType::Explosion => write!(f, "💥 Explosion"),
            EffectType::Rain => write!(f, "🌧️ Rain"),
            EffectType::Snow => write!(f, "❄️ Snow"),
            EffectType::Magic => write!(f, "✨ Magic"),
            EffectType::Energy => write!(f, "⚡ Energy"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

impl Color {
    pub fn new(r: f64, g: f64, b: f64, a: f64) -> Self {
        Self { r, g, b, a }
    }
    
    pub fn to_css_string(&self) -> String {
        format!("rgba({}, {}, {}, {})", self.r, self.g, self.b, self.a)
    }
}

#[derive(Debug, Clone)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub life: f64,
    pub max_life: f64,
    pub size: f64,
    pub color: Color,
    pub gravity: f64,
    pub friction: f64,
    pub fade_rate: f64,
    pub shrink_rate: f64,
    pub angle: f64,
    pub angular_velocity: f64,
    pub particle_type: ParticleType,
    pub color_transitions: Vec<Color>,
    pub trail: Vec<(f64, f64)>,
}

impl Particle {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            x,
            y,
            vx: 0.0,
            vy: 0.0,
            life: 1.0,
            max_life: 1.0,
            size: 3.0,
            color: Color::new(255.0, 255.0, 255.0, 1.0),
            gravity: 0.0,
            friction: 1.0,
            fade_rate: 0.02,
            shrink_rate: 0.0,
            angle: 0.0,
            angular_velocity: 0.0,
            particle_type: ParticleType::Circle,
            color_transitions: Vec::new(),
            trail: Vec::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        // Update position
        self.x += self.vx;
        self.y += self.vy;
        
        // Apply gravity
        self.vy += self.gravity;
        
        // Apply friction
        self.vx *= self.friction;
        self.vy *= self.friction;
        
        // Update angle
        self.angle += self.angular_velocity;
        
        // Update life
        self.life -= self.fade_rate;
        
        // Update size
        self.size -= self.shrink_rate;
        
        // Update color based on life
        if !self.color_transitions.is_empty() {
            let progress = 1.0 - (self.life / self.max_life);
            let color_index = (progress * (self.color_transitions.len() - 1) as f64).floor() as usize;
            if color_index < self.color_transitions.len() {
                self.color = self.color_transitions[color_index].clone();
            }
        }
        
        // Add to trail
        if self.trail.len() > 0 {
            self.trail.push((self.x, self.y));
            if self.trail.len() > 10 {
                self.trail.remove(0);
            }
        }
        
        self.life > 0.0 && self.size > 0.0
    }

    pub fn draw(&self, ctx: &CanvasRenderingContext2d) {
        let alpha = self.life / self.max_life;
        
        // Draw trail
        if self.trail.len() > 1 {
            ctx.save();
            let mut trail_color = self.color.clone();
            trail_color.a = alpha * 0.5;
            ctx.set_stroke_style(&JsValue::from_str(&trail_color.to_css_string()));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(self.trail[0].0, self.trail[0].1);
            for i in 1..self.trail.len() {
                ctx.line_to(self.trail[i].0, self.trail[i].1);
            }
            let _ = ctx.stroke();
            ctx.restore();
        }
        
        ctx.save();
        ctx.set_global_alpha(alpha);
        let _ = ctx.translate(self.x, self.y);
        let _ = ctx.rotate(self.angle);
        
        match self.particle_type {
            ParticleType::Circle => {
                ctx.set_fill_style(&JsValue::from_str(&self.color.to_css_string()));
                ctx.begin_path();
                let _ = ctx.arc(0.0, 0.0, self.size, 0.0, 2.0 * PI);
                ctx.fill();
            }
            ParticleType::Spark => {
                ctx.set_stroke_style(&JsValue::from_str(&self.color.to_css_string()));
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(-self.size, 0.0);
                ctx.line_to(self.size, 0.0);
                ctx.move_to(0.0, -self.size);
                ctx.line_to(0.0, self.size);
                let _ = ctx.stroke();
            }
        }
        
        ctx.restore();
    }
}

pub struct ParticleConfig {
    pub emission_rate: f64,
    pub is_burst: bool,
    pub generator: fn(f64, f64) -> Particle,
}

impl ParticleConfig {
    pub fn get_config(effect_type: &EffectType) -> Self {
        match effect_type {
            EffectType::Fire => ParticleConfig {
                emission_rate: 5.0,
                is_burst: false,
                generator: |x, y| {
                    let mut rng = rand::rng();
                    let mut particle = Particle::new(x + (rng.random::<f64>() - 0.5) * 20.0, y);
                    particle.vx = (rng.random::<f64>() - 0.5) * 2.0;
                    particle.vy = -rng.random::<f64>() * 3.0 - 2.0;
                    particle.size = rng.random::<f64>() * 8.0 + 3.0;
                    particle.gravity = -0.02;
                    particle.friction = 0.98;
                    particle.fade_rate = 0.015;
                    particle.shrink_rate = 0.1;
                    particle.color_transitions = vec![
                        Color::new(255.0, 100.0, 0.0, 1.0),
                        Color::new(255.0, 200.0, 0.0, 1.0),
                        Color::new(255.0, 255.0, 100.0, 0.8),
                        Color::new(100.0, 100.0, 100.0, 0.4),
                        Color::new(50.0, 50.0, 50.0, 0.2),
                    ];
                    particle
                },
            },
            EffectType::Explosion => ParticleConfig {
                emission_rate: 20.0,
                is_burst: true,
                generator: |x, y| {
                    let mut rng = rand::rng();
                    let angle = rng.random::<f64>() * 2.0 * PI;
                    let speed = rng.random::<f64>() * 8.0 + 2.0;
                    let mut particle = Particle::new(x, y);
                    particle.vx = angle.cos() * speed;
                    particle.vy = angle.sin() * speed;
                    particle.size = rng.random::<f64>() * 6.0 + 2.0;
                    particle.gravity = 0.1;
                    particle.friction = 0.95;
                    particle.fade_rate = 0.02;
                    particle.shrink_rate = 0.05;
                    particle.color_transitions = vec![
                        Color::new(255.0, 255.0, 255.0, 1.0),
                        Color::new(255.0, 200.0, 0.0, 1.0),
                        Color::new(255.0, 100.0, 0.0, 0.8),
                        Color::new(255.0, 0.0, 0.0, 0.4),
                        Color::new(100.0, 0.0, 0.0, 0.2),
                    ];
                    particle
                },
            },
            EffectType::Rain => ParticleConfig {
                emission_rate: 8.0,
                is_burst: false,
                generator: |_, _| {
                    let mut rng = rand::rng();
                    let mut particle = Particle::new(rng.random::<f64>() * 800.0, -10.0);
                    particle.vx = -0.5;
                    particle.vy = rng.random::<f64>() * 5.0 + 8.0;
                    particle.size = rng.random::<f64>() * 2.0 + 1.0;
                    particle.gravity = 0.1;
                    particle.friction = 0.999;
                    particle.fade_rate = 0.005;
                    particle.color = Color::new(100.0, 150.0, 255.0, 0.7);
                    particle
                },
            },
            EffectType::Snow => ParticleConfig {
                emission_rate: 3.0,
                is_burst: false,
                generator: |_, _| {
                    let mut rng = rand::rng();
                    let mut particle = Particle::new(rng.random::<f64>() * 800.0, -10.0);
                    particle.vx = (rng.random::<f64>() - 0.5) * 0.5;
                    particle.vy = rng.random::<f64>() * 2.0 + 1.0;
                    particle.size = rng.random::<f64>() * 4.0 + 2.0;
                    particle.gravity = 0.02;
                    particle.friction = 0.999;
                    particle.fade_rate = 0.002;
                    particle.color = Color::new(255.0, 255.0, 255.0, 0.9);
                    particle.angular_velocity = (rng.random::<f64>() - 0.5) * 0.02;
                    particle
                },
            },
            EffectType::Magic => ParticleConfig {
                emission_rate: 2.0,
                is_burst: false,
                generator: |x, y| {
                    let mut rng = rand::rng();
                    let angle = rng.random::<f64>() * 2.0 * PI;
                    let radius = rng.random::<f64>() * 50.0 + 20.0;
                    let mut particle = Particle::new(
                        x + angle.cos() * radius,
                        y + angle.sin() * radius,
                    );
                    particle.vx = (angle + PI / 2.0).cos() * 2.0;
                    particle.vy = (angle + PI / 2.0).sin() * 2.0;
                    particle.size = rng.random::<f64>() * 3.0 + 2.0;
                    particle.friction = 0.99;
                    particle.fade_rate = 0.01;
                    particle.angular_velocity = 0.05;
                    particle.particle_type = ParticleType::Spark;
                    particle.color_transitions = vec![
                        Color::new(255.0, 0.0, 255.0, 1.0),
                        Color::new(0.0, 255.0, 255.0, 1.0),
                        Color::new(255.0, 255.0, 0.0, 0.8),
                        Color::new(255.0, 255.0, 255.0, 0.4),
                    ];
                    particle.trail = vec![];
                    particle
                },
            },
            EffectType::Energy => ParticleConfig {
                emission_rate: 4.0,
                is_burst: false,
                generator: |x, y| {
                    let mut rng = rand::rng();
                    let t = js_sys::Date::now() * 0.001;
                    let spiral_radius = 30.0;
                    let spiral_angle = rng.random::<f64>() * 2.0 * PI;
                    let spiral_x = x + (spiral_angle + t).cos() * spiral_radius;
                    let spiral_y = y + (spiral_angle + t).sin() * spiral_radius;
                    
                    let mut particle = Particle::new(spiral_x, spiral_y);
                    particle.vx = (spiral_angle + t + PI / 2.0).cos() * 1.5;
                    particle.vy = (spiral_angle + t + PI / 2.0).sin() * 1.5;
                    particle.size = rng.random::<f64>() * 4.0 + 2.0;
                    particle.friction = 0.98;
                    particle.fade_rate = 0.015;
                    particle.color_transitions = vec![
                        Color::new(0.0, 255.0, 255.0, 1.0),
                        Color::new(0.0, 150.0, 255.0, 1.0),
                        Color::new(100.0, 100.0, 255.0, 0.8),
                        Color::new(200.0, 200.0, 255.0, 0.4),
                    ];
                    particle.trail = vec![];
                    particle
                },
            },
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct Props {}

pub struct ParticleSystem {
    canvas_ref: NodeRef,
    particles: Vec<Particle>,
    current_effect: EffectType,
    is_emitting: bool,
    mouse_pos: (f64, f64),
    emission_counter: f64,
    _interval: Option<Interval>,
}

pub enum Msg {
    Tick,
    ChangeEffect(EffectType),
    MouseMove(f64, f64),
    MouseDown,
    MouseUp,
    ClearParticles,
}

impl Component for ParticleSystem {
    type Message = Msg;
    type Properties = Props;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            canvas_ref: NodeRef::default(),
            particles: Vec::new(),
            current_effect: EffectType::Fire,
            is_emitting: false,
            mouse_pos: (400.0, 300.0),
            emission_counter: 0.0,
            _interval: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let link = ctx.link().clone();
            let interval = Interval::new(16, move || {
                link.send_message(Msg::Tick);
            });
            self._interval = Some(interval);
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Tick => {
                self.animate();
                true
            }
            Msg::ChangeEffect(effect) => {
                self.current_effect = effect;
                true
            }
            Msg::MouseMove(x, y) => {
                self.mouse_pos = (x, y);
                false
            }
            Msg::MouseDown => {
                self.is_emitting = true;
                false
            }
            Msg::MouseUp => {
                self.is_emitting = false;
                self.emission_counter = 0.0;
                false
            }
            Msg::ClearParticles => {
                self.particles.clear();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let effects = vec![
            EffectType::Fire,
            EffectType::Explosion,
            EffectType::Rain,
            EffectType::Snow,
            EffectType::Magic,
            EffectType::Energy,
        ];

        let onmousemove = ctx.link().callback(|e: MouseEvent| {
            
            let canvas : web_sys::Element  = e.target_unchecked_into();

            let rect = canvas.get_bounding_client_rect();
            Msg::MouseMove(
                e.client_x() as f64 - rect.x(),
                e.client_y() as f64 - rect.y(),
            )
        });

        let onmousedown = ctx.link().callback(|_| Msg::MouseDown);
        let onmouseup = ctx.link().callback(|_| Msg::MouseUp);
        let onmouseleave = ctx.link().callback(|_| Msg::MouseUp);

        html! {
            <div class="flex flex-col items-center space-y-4 p-6 bg-gray-900 min-h-screen">
                <div class="text-center">
                    <h1 class="text-3xl font-bold text-white mb-2">{"Particle System Demo"}</h1>
                    <p class="text-gray-300">{"Click and drag to emit particles"}</p>
                </div>
                
                <div class="flex flex-wrap gap-2 justify-center">
                    { for effects.iter().map(|effect| {
                        let effect_clone = effect.clone();
                        let is_active = *effect == self.current_effect;
                        let onclick = ctx.link().callback(move |_| Msg::ChangeEffect(effect_clone.clone()));
                        html! {
                            <button
                                {onclick}
                                class={format!("px-4 py-2 rounded font-medium transition-colors {}",
                                    if is_active {
                                        "bg-blue-600 text-white"
                                    } else {
                                        "bg-gray-700 text-gray-300 hover:bg-gray-600"
                                    }
                                )}
                            >
                                { effect.to_string() }
                            </button>
                        }
                    }) }
                </div>
                
                <div class="flex gap-4 items-center">
                    <button
                        onclick={ctx.link().callback(|_| Msg::ClearParticles)}
                        class="px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                    >
                        {"Clear Particles"}
                    </button>
                    <div class="text-white">
                        { format!("Particles: {}", self.particles.len()) }
                    </div>
                </div>
                
                <canvas
                    ref={self.canvas_ref.clone()}
                    width="800"
                    height="600"
                    class="border border-gray-600 bg-black cursor-crosshair"
                    {onmousemove}
                    {onmousedown}
                    {onmouseup}
                    {onmouseleave}
                />
                
                <div class="text-gray-400 text-sm max-w-2xl text-center">
                    <p class="mb-2">
                        <strong>{"Current Effect: "}</strong> { self.current_effect.to_string() }
                    </p>
                    <p>
                        {"Move your mouse over the canvas and click to emit particles. "}
                        {"Each effect demonstrates different particle behaviors like gravity, "}
                        {"color transitions, trails, and physics."}
                    </p>
                </div>
            </div>
        }
    }
}

impl ParticleSystem {
    fn animate(&mut self) {
        if let Some(canvas) = self.canvas_ref.cast::<HtmlCanvasElement>() {
            if let Ok(ctx) = canvas.get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<CanvasRenderingContext2d>()
            {
                // Clear canvas with fade effect
                ctx.set_fill_style(&JsValue::from_str("rgba(0, 0, 0, 0.1)"));
                ctx.fill_rect(0.0, 0.0, 800.0, 600.0);
                
                // Update and draw particles
                self.particles.retain_mut(|particle| {
                    let alive = particle.update();
                    if alive {
                        particle.draw(&ctx);
                    }
                    alive
                });
                
                // Emit new particles
                if self.is_emitting {
                    let config = ParticleConfig::get_config(&self.current_effect);
                    self.emission_counter += config.emission_rate;
                    
                    if config.is_burst && self.emission_counter >= config.emission_rate {
                        // Burst emission
                        for _ in 0..(config.emission_rate as usize) {
                            let particle = (config.generator)(self.mouse_pos.0, self.mouse_pos.1);
                            self.particles.push(particle);
                        }
                        self.emission_counter = 0.0;
                    } else if !config.is_burst && self.emission_counter >= 1.0 {
                        // Continuous emission
                        let count = self.emission_counter.floor() as usize;
                        for _ in 0..count {
                            let particle = (config.generator)(self.mouse_pos.0, self.mouse_pos.1);
                            self.particles.push(particle);
                        }
                        self.emission_counter -= count as f64;
                    }
                }
            }
        }
    }
}
