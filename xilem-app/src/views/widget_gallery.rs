use crate::components::{Component, ComponentOutput};
use crate::state::AppState;

/// Hidden widget gallery screen for showcasing components
pub fn widget_gallery() -> ComponentOutput {
    use crate::components::{
        button, card, checkbox, column, domain_card, error_msg, label, labeled_input, 
        loading, metric_display, progress_bar, row, success_msg, text, toast, AppColor
    };
    
    column(vec![
        card(
            "🧪 Hidden Widget Gallery",
            column(vec![
                text("These are experimental/unused components. This page is hidden."),
                
                card(
                    "Basic Components", 
                    column(vec![
                        label("Sample Label"),
                        button("Sample Button", |_state: &mut AppState| {
                            // Demo button action
                        }),
                        progress_bar(0.42, "Upload Progress"),
                        metric_display("Score", "95%", AppColor::Success),
                        row(vec![
                            button("Option A", |_| {}),
                            button("Option B", |_| {}),
                            button("Option C", |_| {}),
                        ]),
                    ])
                ),
                
                card(
                    "Visualization Previews",
                    column(vec![
                        text("Chart components would go here:"),
                        text("• Learning Curve Chart"),
                        text("• Response Time Histogram"),
                        text("• Performance Heatmap"),
                        text("• Metrics Radar Chart"),
                        text("• Progress Ring Chart"),
                        text("• Sparkline"),
                    ])
                ),
                
                card(
                    "Interactive Elements",
                    column(vec![
                        text("Interactive demo elements:"),
                        checkbox(false, "Enable feature", |_state, _checked| {}),
                        labeled_input("Name", "Alice".to_string(), |_state, _value| {}),
                        toast("Success message!", true),
                        loading("Processing..."),
                        error_msg(Some("An error occurred".to_string())),
                        success_msg(Some("Operation completed".to_string())),
                        button("Start Demo", |state: &mut AppState| {
                            if let Some(demo) = &mut state.demo_controller {
                                demo.start_scenario("quick_tour").ok();
                            }
                        }),
                        button("Trigger Easter Egg", |state: &mut AppState| {
                            state.easter_egg_manager.handle_click(0.5, 0.5);
                            state.easter_egg_manager.handle_click(0.5, 0.5);
                            state.easter_egg_manager.handle_click(0.5, 0.5);
                        }),
                    ])
                ),
                
                card(
                    "Domain Cards",
                    column(vec![
                        domain_card("Alphabet", "Learn letter sequences", false),
                        domain_card("Mathematics", "Practice arithmetic", true),
                    ])
                ),
            ])
        ),
        
        button("Back to Dashboard", |state: &mut AppState| {
            state.current_screen = crate::state::Screen::Dashboard;
        }),
    ])
}