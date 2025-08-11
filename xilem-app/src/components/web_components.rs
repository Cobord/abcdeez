// Web implementation of high-level ABCDEEZ components using xilem_web

use xilem_web::AnyDomView;
use xilem_web::elements::html as el;
use xilem_web::interfaces::Element;
use wasm_bindgen::JsCast;
use crate::state::AppState;
use crate::models::Task;
use super::{AppComponents, Component, AppColor, AppScreen, AppTheme};

// Type alias for type-erased web views
type WebDomView = Box<AnyDomView<AppState>>;

// Web component wrapper using xilem_web's AnyDomView
pub struct WebComponent(WebDomView);

impl Component for WebComponent {
    type Output = WebComponent;
    fn build(self) -> Self::Output {
        self
    }
}

pub struct WebComponents;

impl AppComponents for WebComponents {
    type Output = WebComponent;
    
    fn stat_card(title: &str, value: &str, accent_color: AppColor) -> Self::Output {
        let color = format!("color: {}", color_to_css(accent_color));
        WebComponent(Box::new(
            el::div((
                el::h3(title.to_string()),
                el::p(value.to_string()).attr("style", color),
            ))
            .attr("class", "stat-card")
        ))
    }
    
    fn welcome_card(username: &str, message: &str, on_start: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::h2(format!("Welcome, {}!", username)),
                el::p(message.to_string()),
                el::button("Start Learning")
                    .on_click(move |state: &mut AppState, _| {
                        on_start(state);
                    }),
            ))
            .attr("class", "welcome-card")
        ))
    }
    
    fn activity_card(title: &str, time: &str, highlighted: bool) -> Self::Output {
        let class = if highlighted { "activity-card highlighted" } else { "activity-card" };
        WebComponent(Box::new(
            el::div((
                el::h4(title.to_string()),
                el::span(time.to_string()).attr("class", "time"),
            ))
            .attr("class", class)
        ))
    }
    
    fn header_bar(title: &str, on_settings: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        WebComponent(Box::new(
            el::header((
                el::h1(title.to_string()),
                el::button("⚙")
                    .attr("class", "settings-btn")
                    .on_click(move |state: &mut AppState, _| {
                        on_settings(state);
                    }),
            ))
            .attr("class", "header-bar")
        ))
    }
    
    fn bottom_nav_bar(current_screen: AppScreen, on_navigate: impl Fn(&mut AppState, AppScreen) + Send + Sync + 'static) -> Self::Output {
        let on_nav = std::sync::Arc::new(on_navigate);
        
        WebComponent(Box::new(
            el::nav((
                Self::create_nav_btn("Dashboard", AppScreen::Dashboard, current_screen, on_nav.clone()),
                Self::create_nav_btn("Learn", AppScreen::Learning, current_screen, on_nav.clone()),
                Self::create_nav_btn("Progress", AppScreen::Progress, current_screen, on_nav.clone()),
                Self::create_nav_btn("Profile", AppScreen::Profile, current_screen, on_nav.clone()),
            ))
            .attr("class", "bottom-nav")
        ))
    }
    
    fn nav_button(label: &str, screen: AppScreen, is_active: bool, on_click: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        let class = if is_active { "nav-btn active" } else { "nav-btn" };
        WebComponent(Box::new(
            el::button(label.to_string())
                .attr("class", class)
                .on_click(move |state: &mut AppState, _| {
                    on_click(state);
                })
        ))
    }
    
    fn task_presenter(task: &Task, on_answer: impl Fn(&mut AppState, String) + Send + Sync + 'static) -> Self::Output {
        let on_answer = std::sync::Arc::new(on_answer);
        
        WebComponent(Box::new(
            el::div((
                el::h3(task.prompt.clone()),
                el::div(
                    task.options.iter().map(|opt| {
                        let opt_clone = opt.clone();
                        let on_answer_clone = on_answer.clone();
                        el::button(opt.clone())
                            .attr("class", "option-btn")
                            .on_click(move |state: &mut AppState, _| {
                                on_answer_clone(state, opt_clone.clone());
                            })
                    }).collect::<Vec<_>>()
                )
                .attr("class", "options-grid"),
            ))
            .attr("class", "task-presenter")
        ))
    }
    
    fn task_options(options: Vec<String>, on_select: impl Fn(&mut AppState, usize) + Send + Sync + 'static) -> Self::Output {
        let on_select = std::sync::Arc::new(on_select);
        
        WebComponent(Box::new(
            el::div(
                options.iter().enumerate().map(|(idx, opt)| {
                    let on_select_clone = on_select.clone();
                    el::button(opt.clone())
                        .attr("class", "option-btn")
                        .on_click(move |state: &mut AppState, _| {
                            on_select_clone(state, idx);
                        })
                }).collect::<Vec<_>>()
            )
            .attr("class", "task-options")
        ))
    }
    
    fn learning_progress(current: usize, total: usize, accuracy: f64) -> Self::Output {
        let progress_pct = if total > 0 { 
            (current as f64 / total as f64 * 100.0) as u32 
        } else { 
            0 
        };
        
        WebComponent(Box::new(
            el::div((
                el::div((
                    el::span(format!("{}/{}", current, total)),
                    el::span(format!("{}% accurate", (accuracy * 100.0) as u32)),
                ))
                .attr("class", "progress-stats"),
                el::div(
                    el::div("")
                        .attr("class", "progress-fill")
                        .attr("style", format!("width: {}%", progress_pct))
                )
                .attr("class", "progress-bar"),
            ))
            .attr("class", "learning-progress")
        ))
    }
    
    fn hint_button(hint_level: usize, on_hint: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        let label = match hint_level {
            0 => "Get Hint",
            1 => "More Hints",
            _ => "Show Answer",
        };
        
        WebComponent(Box::new(
            el::button(label.to_string())
                .attr("class", "hint-btn")
                .on_click(move |state: &mut AppState, _| {
                    on_hint(state);
                })
        ))
    }
    
    fn alphabet_sequence(items: Vec<String>, highlight_index: Option<usize>) -> Self::Output {
        WebComponent(Box::new(
            el::div(
                items.iter().enumerate().map(|(idx, item)| {
                    let class = if Some(idx) == highlight_index {
                        "alphabet-item highlighted"
                    } else {
                        "alphabet-item"
                    };
                    el::span(item.clone()).attr("class", class)
                }).collect::<Vec<_>>()
            )
            .attr("class", "alphabet-sequence")
        ))
    }
    
    fn comparison_visual(left: &str, right: &str, show_order: bool) -> Self::Output {
        let arrow = if show_order { "→" } else { "?" };
        WebComponent(Box::new(
            el::div((
                el::span(left.to_string()).attr("class", "comparison-item"),
                el::span(arrow.to_string()).attr("class", "comparison-arrow"),
                el::span(right.to_string()).attr("class", "comparison-item"),
            ))
            .attr("class", "comparison-visual")
        ))
    }
    
    fn missing_item_visual(before: &str, after: &str) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::span(before.to_string()).attr("class", "sequence-item"),
                el::span("?".to_string()).attr("class", "missing-item"),
                el::span(after.to_string()).attr("class", "sequence-item"),
            ))
            .attr("class", "missing-item-visual")
        ))
    }
    
    fn path_visual(start: &str, end: &str, path: Vec<String>) -> Self::Output {
        let path_str = path.join(" → ");
        WebComponent(Box::new(
            el::div((
                el::span(start.to_string()).attr("class", "path-node start"),
                el::span(path_str).attr("class", "path-steps"),
                el::span(end.to_string()).attr("class", "path-node end"),
            ))
            .attr("class", "path-visual")
        ))
    }
    
    fn simple_label(text: String) -> Self::Output {
        WebComponent(Box::new(
            el::span(text)
        ))
    }
    
    fn simple_flex_column(items: Vec<Self::Output>) -> Self::Output {
        let views: Vec<WebDomView> = items.into_iter()
            .map(|item| item.0)
            .collect();
        
        WebComponent(Box::new(
            el::div(views).attr("class", "flex-column")
        ))
    }
    
    fn simple_flex_row(items: Vec<Self::Output>) -> Self::Output {
        let views: Vec<WebDomView> = items.into_iter()
            .map(|item| item.0)
            .collect();
        
        WebComponent(Box::new(
            el::div(views).attr("class", "flex-row")
        ))
    }
    
    fn simple_button(text: &str, on_click: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        WebComponent(Box::new(
            el::button(text.to_string())
                .attr("class", "btn")
                .on_click(move |state: &mut AppState, _| {
                    on_click(state);
                })
        ))
    }
    
    fn auth_form(
        username: String,
        password: String,
        on_username: impl Fn(&mut AppState, String) + Send + Sync + 'static,
        on_password: impl Fn(&mut AppState, String) + Send + Sync + 'static,
        on_submit: impl Fn(&mut AppState) + Send + Sync + 'static,
    ) -> Self::Output {
        WebComponent(Box::new(
            el::form((
                el::input(())
                    .attr("type", "text")
                    .attr("placeholder", "Username")
                    .attr("value", username)
                    .on_input(move |state: &mut AppState, evt: web_sys::Event| {
                        if let Some(input) = evt.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                            on_username(state, input.value());
                        }
                    }),
                el::input(())
                    .attr("type", "password")
                    .attr("placeholder", "Password")
                    .attr("value", password)
                    .on_input(move |state: &mut AppState, evt: web_sys::Event| {
                        if let Some(input) = evt.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                            on_password(state, input.value());
                        }
                    }),
                el::button("Sign In")
                    .attr("type", "submit")
                    .on_click(move |state: &mut AppState, _| {
                        on_submit(state);
                    }),
            ))
            .attr("class", "auth-form")
            .on_submit(move |state: &mut AppState, evt: web_sys::Event| {
                evt.prevent_default();
            })
        ))
    }
    
    fn settings_section(title: &str, children: Vec<Self::Output>) -> Self::Output {
        let mut views: Vec<WebDomView> = Vec::new();
        
        if !title.is_empty() {
            views.push(Box::new(el::h3(title.to_string())));
        }
        
        views.extend(children.into_iter().map(|c| c.0));
        
        WebComponent(Box::new(
            el::section(views).attr("class", "settings-section")
        ))
    }
    
    fn setting_row(label: &str, control: Self::Output) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::label(label.to_string()),
                control.0,
            ))
            .attr("class", "setting-row")
        ))
    }
    
    fn theme_selector(current: AppTheme, on_change: impl Fn(&mut AppState, AppTheme) + Send + Sync + 'static) -> Self::Output {
        let on_change = std::sync::Arc::new(on_change);
        
        // For now, just create without selected attribute - the select will handle the value
        WebComponent(Box::new(
            el::select((
                el::option("Light").attr("value", "light"),
                el::option("Dark").attr("value", "dark"),
                el::option("Auto").attr("value", "auto"),
            ))
            .on_change(move |state: &mut AppState, evt: web_sys::Event| {
                if let Some(select) = evt.target().and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok()) {
                    let theme = match select.value().as_str() {
                        "dark" => AppTheme::Dark,
                        "auto" => AppTheme::Auto,
                        _ => AppTheme::Light,
                    };
                    on_change(state, theme);
                }
            })
            .attr("class", "theme-selector")
        ))
    }
    
    fn app_scaffold(header: Self::Output, content: Self::Output, footer: Option<Self::Output>) -> Self::Output {
        let mut views: Vec<WebDomView> = vec![
            header.0,
            Box::new(el::main(content.0).attr("class", "app-content")),
        ];
        
        if let Some(footer) = footer {
            views.push(footer.0);
        }
        
        WebComponent(Box::new(
            el::div(views).attr("class", "app-scaffold")
        ))
    }
    
    fn centered_container(max_width: f64, child: Self::Output) -> Self::Output {
        WebComponent(Box::new(
            el::div(child.0)
                .attr("class", "centered-container")
                .attr("style", format!("max-width: {}px", max_width))
        ))
    }
    
    fn stats_grid(stats: Vec<Self::Output>) -> Self::Output {
        let views: Vec<WebDomView> = stats.into_iter()
            .map(|s| s.0)
            .collect();
        
        WebComponent(Box::new(
            el::div(views).attr("class", "stats-grid")
        ))
    }
    
    fn loading_spinner(message: Option<&str>) -> Self::Output {
        let mut views: Vec<WebDomView> = vec![
            Box::new(el::div("").attr("class", "spinner")),
        ];
        
        if let Some(msg) = message {
            views.push(Box::new(el::p(msg.to_string())));
        }
        
        WebComponent(Box::new(
            el::div(views).attr("class", "loading-spinner")
        ))
    }
    
    fn empty_state(icon: &str, title: &str, message: &str, action: Option<Self::Output>) -> Self::Output {
        let mut views: Vec<WebDomView> = vec![
            Box::new(el::div(icon.to_string()).attr("class", "empty-icon")),
            Box::new(el::h3(title.to_string())),
            Box::new(el::p(message.to_string())),
        ];
        
        if let Some(action) = action {
            views.push(action.0);
        }
        
        WebComponent(Box::new(
            el::div(views).attr("class", "empty-state")
        ))
    }
    
    fn error_banner(message: &str, on_dismiss: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::span(message.to_string()),
                el::button("×")
                    .attr("class", "dismiss-btn")
                    .on_click(move |state: &mut AppState, _| {
                        on_dismiss(state);
                    }),
            ))
            .attr("class", "error-banner")
        ))
    }
    
    // Additional ported components from old app
    fn card<V>(title: &str, content: V) -> Self::Output where V: Component<Output = Self::Output> {
        WebComponent(Box::new(
            el::div((
                el::h3(title.to_string()),
                content.build().0,
            ))
            .attr("class", "card")
        ))
    }
    
    fn progress_bar(progress: f64, label: &str) -> Self::Output {
        let percentage = (progress * 100.0) as u32;
        WebComponent(Box::new(
            el::div((
                el::label(label.to_string()),
                el::div(
                    el::div("")
                        .attr("class", "progress-fill")
                        .attr("style", format!("width: {}%", percentage))
                )
                .attr("class", "progress-bar"),
                el::span(format!("{}%", percentage)),
            ))
            .attr("class", "progress-container")
        ))
    }
    
    fn metric_display(label: &str, value: &str, color: AppColor) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::label(label.to_string()).attr("class", "metric-label"),
                el::span(value.to_string())
                    .attr("class", "metric-value")
                    .attr("style", format!("color: {}", color_to_css(color))),
            ))
            .attr("class", "metric-display")
        ))
    }
    
    fn checkbox(checked: bool, label: &str, on_toggle: impl Fn(&mut AppState, bool) + Send + Sync + 'static) -> Self::Output {
        WebComponent(Box::new(
            el::label((
                el::input(())
                    .attr("type", "checkbox")
                    .attr("checked", if checked { "checked" } else { "" })
                    .on_change(move |state: &mut AppState, evt: web_sys::Event| {
                        if let Some(input) = evt.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                            on_toggle(state, input.checked());
                        }
                    }),
                el::span(label.to_string()),
            ))
            .attr("class", "checkbox-wrapper")
        ))
    }
    
    fn labeled_input(label: &str, value: String, on_change: impl Fn(&mut AppState, String) + Send + Sync + 'static) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::label(label.to_string()),
                el::input(())
                    .attr("type", "text")
                    .attr("value", value)
                    .on_input(move |state: &mut AppState, evt: web_sys::Event| {
                        if let Some(input) = evt.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                            on_change(state, input.value());
                        }
                    }),
            ))
            .attr("class", "labeled-input")
        ))
    }
    
    fn toast_notification(message: &str, is_success: bool) -> Self::Output {
        let class = if is_success { "toast success" } else { "toast error" };
        WebComponent(Box::new(
            el::div(message.to_string())
                .attr("class", class)
        ))
    }
    
    fn loading_overlay(message: &str) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::div("").attr("class", "spinner"),
                el::p(message.to_string()),
            ))
            .attr("class", "loading-overlay")
        ))
    }
    
    fn nav_bar(current_screen: &str) -> Self::Output {
        WebComponent(Box::new(
            el::nav(
                el::span(format!("📍 {}", current_screen))
            )
            .attr("class", "nav-bar")
        ))
    }
    
    fn error_message(message: Option<String>) -> Self::Output {
        if let Some(msg) = message {
            WebComponent(Box::new(
                el::div(format!("❌ {}", msg))
                    .attr("class", "error-message")
            ))
        } else {
            WebComponent(Box::new(el::div("")))
        }
    }
    
    fn success_message(message: Option<String>) -> Self::Output {
        if let Some(msg) = message {
            WebComponent(Box::new(
                el::div(format!("✅ {}", msg))
                    .attr("class", "success-message")
            ))
        } else {
            WebComponent(Box::new(el::div("")))
        }
    }
    
    fn highlighted<V>(element_id: &str, content: V, has_highlight: bool, tooltip: Option<String>) -> Self::Output 
        where V: Component<Output = Self::Output> {
        let class = if has_highlight { "highlighted" } else { "" };
        let element_id_owned = element_id.to_string();
        
        let mut views: Vec<WebDomView> = vec![content.build().0];
        
        if let Some(tip) = tooltip {
            views.push(Box::new(
                el::div(format!("💡 {}", tip))
                    .attr("class", "tooltip")
            ));
        }
        
        WebComponent(Box::new(
            el::div(views)
                .attr("class", class)
                .attr("data-element-id", element_id_owned)
        ))
    }
    
    fn demo_button(element_id: &str, text: &str, has_highlight: bool, tooltip: Option<String>) -> Self::Output {
        let button_view = Self::simple_button(text, |_| {});
        Self::highlighted(element_id, button_view, has_highlight, tooltip)
    }
    
    fn demo_card<V>(element_id: &str, title: &str, content: V, has_highlight: bool, tooltip: Option<String>) -> Self::Output
        where V: Component<Output = Self::Output> {
        let card_view = Self::card(title, content);
        Self::highlighted(element_id, card_view, has_highlight, tooltip)
    }
    
    fn confirm_modal(
        title: &str,
        message: &str,
        on_confirm: impl Fn(&mut AppState) + Send + Sync + 'static,
        on_cancel: impl Fn(&mut AppState) + Send + Sync + 'static,
    ) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::h3(title.to_string()),
                el::p(message.to_string()),
                el::div((
                    el::button("Cancel")
                        .attr("class", "btn-cancel")
                        .on_click(move |state: &mut AppState, _| {
                            on_cancel(state);
                        }),
                    el::button("Confirm")
                        .attr("class", "btn-confirm")
                        .on_click(move |state: &mut AppState, _| {
                            on_confirm(state);
                        }),
                ))
                .attr("class", "modal-actions"),
            ))
            .attr("class", "confirm-modal")
        ))
    }
    
    fn response_time_histogram(response_times: &[u128]) -> Self::Output {
        let avg = if !response_times.is_empty() {
            response_times.iter().sum::<u128>() / response_times.len() as u128
        } else {
            0
        };
        
        WebComponent(Box::new(
            el::div((
                el::h4("Response Time Histogram"),
                el::p(format!("📊 Average: {}ms", avg)),
                el::p(format!("{} samples", response_times.len())),
            ))
            .attr("class", "histogram")
        ))
    }
    
    fn learning_curve_display(responses: &[crate::models::Response]) -> Self::Output {
        let correct = responses.iter().filter(|r| r.correct).count();
        let total = responses.len();
        
        WebComponent(Box::new(
            el::div((
                el::h4("Learning Curve"),
                el::p(format!("📈 {}/{} correct", correct, total)),
            ))
            .attr("class", "learning-curve")
        ))
    }
    
    fn error_analysis_display(responses: &[crate::models::Response]) -> Self::Output {
        let errors = responses.iter().filter(|r| !r.correct).count();
        
        WebComponent(Box::new(
            el::div((
                el::h4("Error Analysis"),
                el::p(format!("❌ {} errors found", errors))
                    .attr("class", "error-count"),
            ))
            .attr("class", "error-analysis")
        ))
    }
    
    fn strategy_analysis_display(strategies: Vec<(&str, f64)>) -> Self::Output {
        let strategy_items: Vec<WebDomView> = strategies.into_iter()
            .take(3)
            .map(|(name, score)| {
                Box::new(
                    el::li(format!("{}: {:.0}%", name, score * 100.0))
                ) as WebDomView
            })
            .collect();
        
        WebComponent(Box::new(
            el::div((
                el::h4("Strategy Analysis"),
                el::ul(strategy_items),
            ))
            .attr("class", "strategy-analysis")
        ))
    }
    
    fn domain_card(domain: &str, description: &str, selected: bool) -> Self::Output {
        let class = if selected { "domain-card selected" } else { "domain-card" };
        
        WebComponent(Box::new(
            el::div((
                el::h4(domain.to_string()),
                el::p(description.to_string()),
            ))
            .attr("class", class)
        ))
    }
    
    fn session_info(session_id: &str, status: &str, duration: &str, domain: &str) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::h5(format!("Session: {}", session_id)),
                el::div((
                    el::span(format!("Status: {}", status)),
                    el::span(format!("Duration: {}", duration)),
                    el::span(format!("Domain: {}", domain)),
                ))
                .attr("class", "session-details"),
            ))
            .attr("class", "session-info")
        ))
    }
    
    fn task_card_display(prompt: &str, hint: Option<&str>) -> Self::Output {
        let mut views: Vec<WebDomView> = vec![
            Box::new(el::h3(prompt.to_string())),
        ];
        
        if let Some(hint_text) = hint {
            views.push(Box::new(
                el::p(format!("💡 Hint: {}", hint_text))
                    .attr("class", "hint")
            ));
        }
        
        WebComponent(Box::new(
            el::div(views).attr("class", "task-card")
        ))
    }
    
    fn answer_options_display(options: Vec<String>, on_select: impl Fn(&mut AppState, usize) + Send + Sync + 'static) -> Self::Output {
        let on_select = std::sync::Arc::new(on_select);
        
        let buttons: Vec<WebDomView> = options.into_iter()
            .enumerate()
            .map(|(idx, opt)| {
                let on_select_clone = on_select.clone();
                Box::new(
                    el::button(opt)
                        .attr("class", "answer-option")
                        .on_click(move |state: &mut AppState, _| {
                            on_select_clone(state, idx);
                        })
                ) as WebDomView
            })
            .collect();
        
        WebComponent(Box::new(
            el::div(buttons).attr("class", "answer-options")
        ))
    }
    
    fn performance_chart(accuracy: f64, total_responses: usize, correct: usize, avg_time: f64) -> Self::Output {
        WebComponent(Box::new(
            el::div((
                el::h4("Performance"),
                el::div((
                    Self::metric_display("Accuracy", &format!("{:.0}%", accuracy * 100.0), AppColor::Success).0,
                    Self::metric_display("Total", &total_responses.to_string(), AppColor::Info).0,
                ))
                .attr("class", "metrics-row"),
                el::div((
                    Self::metric_display("Correct", &correct.to_string(), AppColor::Success).0,
                    Self::metric_display("Avg Time", &format!("{:.0}ms", avg_time), AppColor::Info).0,
                ))
                .attr("class", "metrics-row"),
            ))
            .attr("class", "performance-chart")
        ))
    }
}

// Helper functions
impl WebComponents {
    fn create_nav_btn(
        label: &str, 
        screen: AppScreen, 
        current: AppScreen,
        on_nav: std::sync::Arc<dyn Fn(&mut AppState, AppScreen) + Send + Sync>
    ) -> WebDomView {
        let is_active = screen == current;
        let class = if is_active { "nav-btn active" } else { "nav-btn" };
        
        Box::new(
            el::button(label.to_string())
                .attr("class", class)
                .on_click(move |state: &mut AppState, _| {
                    on_nav(state, screen);
                })
        )
    }
}

fn color_to_css(color: AppColor) -> &'static str {
    match color {
        AppColor::Primary => "#0066cc",
        AppColor::Secondary => "#6c757d",
        AppColor::Success => "#28a745",
        AppColor::Warning => "#ffc107",
        AppColor::Error => "#dc3545",
        AppColor::Info => "#17a2b8",
        AppColor::Surface => "#f8f9fa",
        AppColor::Background => "#ffffff",
        AppColor::Text => "#212529",
        AppColor::TextMuted => "#6c757d",
    }
}

