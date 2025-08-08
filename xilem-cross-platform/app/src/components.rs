use xilem::{
    view::{button, flex, label, prose, textbox, Axis},
    Color, TextAlignment, WidgetView,
};

use crate::models::*;
use crate::AppData;

// Reusable card component
pub fn card<T, V>(title: &str, content: V) -> impl WidgetView<T>
where
    V: WidgetView<T>,
{
    flex((
        label(title)
            .brush(Color::from_rgb8(64, 64, 64))
            .alignment(TextAlignment::Start),
        content,
    ))
    .direction(Axis::Vertical)
}

// Progress bar component
pub fn progress_bar(progress: f64, label: String) -> impl WidgetView<AppData> {
    let width = 300.0;
    let height = 20.0;
    let filled_width = (width * progress.min(1.0).max(0.0)) as i32;
    
    flex((
        label(label.clone())
            .alignment(TextAlignment::Middle),
        // Simple text-based progress visualization
        label(format!("[{}{}] {:.1}%", 
            "=".repeat((filled_width / 10) as usize),
            " ".repeat(((width - filled_width as f64) / 10.0) as usize),
            progress * 100.0
        ))
        .brush(Color::from_rgb8(0, 128, 255))
        .alignment(TextAlignment::Middle),
    ))
    .direction(Axis::Vertical)
}

// Metric display component
pub fn metric_display(label_text: &str, value: String, color: Color) -> impl WidgetView<AppData> {
    flex((
        label(label_text)
            .brush(Color::from_rgb8(128, 128, 128))
            .alignment(TextAlignment::Start),
        label(value)
            .brush(color)
            .alignment(TextAlignment::End),
    ))
    .direction(Axis::Horizontal)
}

// Task card component for displaying questions
pub fn task_card(task: &Task) -> impl WidgetView<AppData> {
    match task {
        Task::Alphabet(alphabet_task) => {
            flex((
                label(format!("What position is '{}' in the alphabet?", alphabet_task.letter))
                    .alignment(TextAlignment::Middle),
                label("Select your answer below")
                    .brush(Color::from_rgb8(128, 128, 128))
                    .alignment(TextAlignment::Middle),
            ))
            .direction(Axis::Vertical)
        },
        Task::Music(music_task) => {
            flex((
                label(&music_task.prompt)
                    .alignment(TextAlignment::Middle),
                label(&music_task.task_type)
                    .brush(Color::from_rgb8(128, 128, 128))
                    .alignment(TextAlignment::Middle),
            ))
            .direction(Axis::Vertical)
        },
        Task::Custom(_) => {
            label("Custom task")
                .alignment(TextAlignment::Middle)
        }
    }
}

// Answer options component
pub fn answer_options(options: Vec<String>, on_select: impl Fn(&mut AppData, usize) + 'static + Clone) -> impl WidgetView<AppData> {
    let buttons = options
        .into_iter()
        .enumerate()
        .map(move |(index, option)| {
            let on_select = on_select.clone();
            button(option, move |data: &mut AppData| {
                on_select(data, index);
            })
        })
        .collect::<Vec<_>>();
    
    flex(buttons)
        .direction(Axis::Vertical)
}

// Performance chart component (simplified text-based)
pub fn performance_chart(metrics: &PerformanceMetrics) -> impl WidgetView<AppData> {
    flex((
        label("Performance Overview")
            .brush(Color::from_rgb8(64, 64, 64))
            .alignment(TextAlignment::Middle),
        metric_display(
            "Accuracy:", 
            format!("{:.1}%", metrics.accuracy_rate * 100.0),
            Color::from_rgb8(0, 200, 0)
        ),
        metric_display(
            "Total Responses:", 
            metrics.total_responses.to_string(),
            Color::from_rgb8(0, 128, 255)
        ),
        metric_display(
            "Correct:", 
            metrics.correct_responses.to_string(),
            Color::from_rgb8(0, 200, 0)
        ),
        metric_display(
            "Avg Response Time:", 
            format!("{:.0}ms", metrics.average_response_time_ms),
            Color::from_rgb8(255, 128, 0)
        ),
    ))
    .direction(Axis::Vertical)
}

// Domain card component
pub fn domain_card(domain: &Domain, selected: bool) -> impl WidgetView<AppData> {
    let color = if selected {
        Color::from_rgb8(0, 128, 255)
    } else {
        Color::from_rgb8(128, 128, 128)
    };
    
    flex((
        label(domain.display_name())
            .brush(color)
            .alignment(TextAlignment::Middle),
        label(match domain {
            Domain::Alphabet => "Learn letter positions and sequences",
            Domain::Music => "Master intervals, scales, and theory",
            Domain::Mathematics => "Practice arithmetic and patterns",
            Domain::Custom(_) => "Custom learning domain",
        })
        .brush(Color::from_rgb8(160, 160, 160))
        .alignment(TextAlignment::Middle),
    ))
    .direction(Axis::Vertical)
}

// Session info component
pub fn session_info(session: &Session) -> impl WidgetView<AppData> {
    let duration = if let Some(end) = session.end_time {
        let diff = end - session.start_time;
        format!("{}m {}s", diff.num_minutes(), diff.num_seconds() % 60)
    } else {
        "In Progress".to_string()
    };
    
    flex((
        metric_display("Session ID:", &session.id[..8], Color::from_rgb8(128, 128, 128)),
        metric_display("Status:", &session.status, Color::from_rgb8(0, 200, 0)),
        metric_display("Duration:", duration, Color::from_rgb8(0, 128, 255)),
        metric_display("Domain:", &session.topology_type, Color::from_rgb8(255, 128, 0)),
    ))
    .direction(Axis::Vertical)
}

// Input field with label
pub fn labeled_input(
    label_text: &str, 
    value: String, 
    on_change: impl Fn(&mut AppData, String) + 'static
) -> impl WidgetView<AppData> {
    flex((
        label(label_text)
            .alignment(TextAlignment::Start),
        textbox(value, on_change),
    ))
    .direction(Axis::Vertical)
}

// Navigation button bar
pub fn nav_bar(current_screen: &str) -> impl WidgetView<AppData> {
    flex((
        button("Home", |data: &mut AppData| {
            data.current_screen = crate::Screen::Welcome;
        }),
        button("Domains", |data: &mut AppData| {
            data.current_screen = crate::Screen::DomainSelection;
        }),
        button("Training", |data: &mut AppData| {
            if data.current_session.is_some() {
                data.current_screen = crate::Screen::Training;
            }
        }),
        button("Dashboard", |data: &mut AppData| {
            data.current_screen = crate::Screen::Dashboard;
        }),
        button("Export", |data: &mut AppData| {
            data.current_screen = crate::Screen::Export;
        }),
    ))
    .direction(Axis::Horizontal)
}

// Error message display
pub fn error_message(message: Option<String>) -> impl WidgetView<AppData> {
    if let Some(msg) = message {
        label(msg)
            .brush(Color::from_rgb8(255, 0, 0))
            .alignment(TextAlignment::Middle)
    } else {
        label("")
            .alignment(TextAlignment::Middle)
    }
}

// Success message display
pub fn success_message(message: Option<String>) -> impl WidgetView<AppData> {
    if let Some(msg) = message {
        label(msg)
            .brush(Color::from_rgb8(0, 200, 0))
            .alignment(TextAlignment::Middle)
    } else {
        label("")
            .alignment(TextAlignment::Middle)
    }
}