use wasm_bindgen::prelude::*;
use web_sys::{Document, Element, HtmlElement, window};
use abcdeez_core::{
    learning::adaptive::AdaptiveScheduler,
    learning::learner::LearnerModel,
    tasks::core::{Task, TaskGenerator},
    core::topology::Topology,
};
use serde::{Deserialize, Serialize};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    current_screen: String,
    selected_domain: Option<String>,
    total_trials: usize,
    correct_trials: usize,
    current_task: Option<String>,
    current_target: Option<char>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            current_screen: "welcome".to_string(),
            selected_domain: None,
            total_trials: 0,
            correct_trials: 0,
            current_task: None,
            current_target: None,
        }
    }
}

#[wasm_bindgen]
pub struct GraphLearningApp {
    state: AppState,
    scheduler: Option<AdaptiveScheduler>,
    topology: Option<Topology>,
    task_generator: Option<TaskGenerator>,
    current_task: Option<Task>,
    document: Document,
}

#[wasm_bindgen]
impl GraphLearningApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<GraphLearningApp, JsValue> {
        console_error_panic_hook::set_once();
        
        let window = window().unwrap();
        let document = window.document().unwrap();
        
        let mut app = GraphLearningApp {
            state: AppState::default(),
            scheduler: None,
            topology: None,
            task_generator: None,
            current_task: None,
            document,
        };
        
        // Load state from localStorage if available
        app.load_state();
        
        Ok(app)
    }
    
    pub fn init(&mut self) -> Result<(), JsValue> {
        console_log!("Initializing Graph Learning App");
        self.render()?;
        Ok(())
    }
    
    pub fn render(&mut self) -> Result<(), JsValue> {
        let app_element = self.document
            .get_element_by_id("app")
            .ok_or("Could not find #app element")?;
        
        // Clear existing content
        app_element.set_inner_html("");
        
        match self.state.current_screen.as_str() {
            "welcome" => self.render_welcome_screen(&app_element)?,
            "domains" => self.render_domain_selection(&app_element)?,
            "training" => self.render_training_screen(&app_element)?,
            "dashboard" => self.render_dashboard(&app_element)?,
            _ => self.render_welcome_screen(&app_element)?,
        }
        
        Ok(())
    }
    
    fn render_welcome_screen(&self, container: &Element) -> Result<(), JsValue> {
        let html = r#"
            <div class="container">
                <div class="card">
                    <h1>🎓 Welcome to Graph Learning</h1>
                    <p>An adaptive learning system for cognitive skill acquisition</p>
                    <div class="button-group">
                        <button id="start-btn" class="button">Start Learning</button>
                        <button id="dashboard-btn" class="button secondary">View Dashboard</button>
                    </div>
                </div>
            </div>
        "#;
        
        container.set_inner_html(html);
        
        // Add event listeners
        if let Some(start_btn) = self.document.get_element_by_id("start-btn") {
            let start_btn = start_btn.dyn_into::<HtmlElement>()?;
            let closure = Closure::wrap(Box::new(move || {
                console_log!("Start button clicked");
            }) as Box<dyn Fn()>);
            start_btn.set_onclick(Some(closure.as_ref().unchecked_ref()));
            closure.forget();
        }
        
        Ok(())
    }
    
    fn render_domain_selection(&self, container: &Element) -> Result<(), JsValue> {
        let html = r#"
            <div class="container">
                <div class="card">
                    <h2>Select Learning Domain</h2>
                    <div class="domain-grid">
                        <div class="domain-card" data-domain="alphabet">
                            <h3>🔤 Alphabet</h3>
                            <p>Learn letter sequences</p>
                        </div>
                        <div class="domain-card" data-domain="numbers">
                            <h3>🔢 Numbers</h3>
                            <p>Practice numerical patterns</p>
                        </div>
                        <div class="domain-card" data-domain="music">
                            <h3>🎵 Music</h3>
                            <p>Musical note training</p>
                        </div>
                        <div class="domain-card" data-domain="days">
                            <h3>📅 Days</h3>
                            <p>Days of the week</p>
                        </div>
                    </div>
                </div>
            </div>
        "#;
        
        container.set_inner_html(html);
        Ok(())
    }
    
    fn render_training_screen(&self, container: &Element) -> Result<(), JsValue> {
        let task_display = self.state.current_task.as_deref().unwrap_or("Press Start");
        let accuracy = if self.state.total_trials > 0 {
            (self.state.correct_trials as f64 / self.state.total_trials as f64) * 100.0
        } else {
            0.0
        };
        
        let html = format!(r#"
            <div class="container">
                <div class="card">
                    <h2>Training: {}</h2>
                    <div class="task-display">{}</div>
                    <div class="response-grid">
                        {} 
                    </div>
                    <div class="progress">
                        <div>Accuracy: {:.1}%</div>
                        <div class="progress-bar">
                            <div class="progress-fill" style="width: {:.1}%"></div>
                        </div>
                    </div>
                </div>
            </div>
        "#, 
            self.state.selected_domain.as_deref().unwrap_or("None"),
            task_display,
            self.generate_response_buttons(),
            accuracy,
            accuracy
        );
        
        container.set_inner_html(&html);
        Ok(())
    }
    
    fn render_dashboard(&self, container: &Element) -> Result<(), JsValue> {
        let accuracy = if self.state.total_trials > 0 {
            (self.state.correct_trials as f64 / self.state.total_trials as f64) * 100.0
        } else {
            0.0
        };
        
        let html = format!(r#"
            <div class="container">
                <div class="card">
                    <h2>Performance Dashboard</h2>
                    <div class="metrics">
                        <div class="metric-card">
                            <div class="metric-value">{}</div>
                            <div class="metric-label">Total Trials</div>
                        </div>
                        <div class="metric-card">
                            <div class="metric-value">{}</div>
                            <div class="metric-label">Correct</div>
                        </div>
                        <div class="metric-card">
                            <div class="metric-value">{:.1}%</div>
                            <div class="metric-label">Accuracy</div>
                        </div>
                    </div>
                </div>
            </div>
        "#, 
            self.state.total_trials,
            self.state.correct_trials,
            accuracy
        );
        
        container.set_inner_html(&html);
        Ok(())
    }
    
    fn generate_response_buttons(&self) -> String {
        ('A'..='Z').map(|c| {
            format!(r#"<button class="response-btn" data-response="{}">{}</button>"#, c, c)
        }).collect::<Vec<_>>().join("")
    }
    
    pub fn navigate_to(&mut self, screen: &str) -> Result<(), JsValue> {
        self.state.current_screen = screen.to_string();
        self.save_state();
        self.render()?;
        Ok(())
    }
    
    pub fn start_training(&mut self, domain: &str) -> Result<(), JsValue> {
        console_log!("Starting training for domain: {}", domain);
        
        self.state.selected_domain = Some(domain.to_string());
        
        // Initialize topology and scheduler
        let topology = Topology::alphabet();
        let learner_model = LearnerModel::new("web_user".to_string(), &topology);
        self.scheduler = Some(AdaptiveScheduler::new(learner_model, topology.clone()));
        self.task_generator = Some(TaskGenerator::new(topology.clone()));
        self.topology = Some(topology);
        
        self.generate_next_task();
        self.state.current_screen = "training".to_string();
        
        self.save_state();
        self.render()?;
        
        Ok(())
    }
    
    fn generate_next_task(&mut self) {
        if let Some(generator) = &mut self.task_generator {
            let task = generator.generate_task(None);
            self.state.current_task = Some(task.prompt.clone());
            self.state.current_target = Some(task.correct_answer.chars().next().unwrap_or('A'));
            self.current_task = Some(task);
        }
    }
    
    pub fn submit_response(&mut self, response: char) -> Result<(), JsValue> {
        if let Some(task) = &self.current_task {
            let response_str = response.to_string();
            let correct = task.correct_answer == response_str;
            
            self.state.total_trials += 1;
            if correct {
                self.state.correct_trials += 1;
            }
            
            // Update scheduler with response
            if let Some(scheduler) = &mut self.scheduler {
                scheduler.update_after_response(task, correct, 1.5);
            }
            
            self.generate_next_task();
            self.save_state();
            self.render()?;
        }
        
        Ok(())
    }
    
    fn save_state(&self) {
        if let Ok(storage) = window().unwrap().local_storage() {
            if let Some(storage) = storage {
                if let Ok(json) = serde_json::to_string(&self.state) {
                    let _ = storage.set_item("graph_learning_state", &json);
                }
            }
        }
    }
    
    fn load_state(&mut self) {
        if let Ok(storage) = window().unwrap().local_storage() {
            if let Some(storage) = storage {
                if let Ok(Some(json)) = storage.get_item("graph_learning_state") {
                    if let Ok(state) = serde_json::from_str(&json) {
                        self.state = state;
                    }
                }
            }
        }
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("Graph Learning WASM Module Loaded");
}