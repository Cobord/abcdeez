// Native implementation of high-level ABCDEEZ components

use xilem::style::Style;
use xilem::view::*;
use xilem::{AnyWidgetView, Color};
use crate::state::{AppState, Screen};
use crate::models::Task;
use super::{AppComponents, Component, AppColor, AppTheme};

// Native component wrapper
pub struct NativeComponent(pub Box<AnyWidgetView<AppState>>);

impl Component for NativeComponent {
    type Output = NativeComponent;
    fn build(self) -> Self::Output {
        self
    }
}

pub struct NativeComponents;

impl AppComponents for NativeComponents {
    type Output = NativeComponent;
    
    fn stat_card(title: &str, value: &str, accent_color: AppColor) -> Self::Output {
        let color = map_color(accent_color);
        NativeComponent(Box::new(
            sized_box(
                flex((
                    label(title)
                        .text_size(12.0)
                        .color(map_color(AppColor::TextMuted)),
                    FlexSpacer::Fixed(5.0),
                    label(value)
                        .text_size(24.0)
                        .weight(xilem::FontWeight::BOLD)
                        .color(color),
                ))
                .direction(Axis::Vertical)
                .cross_axis_alignment(CrossAxisAlignment::Center)
                .padding(15.0)
            )
            .background_color(Color::from_rgb8(245, 245, 250))
            .corner_radius(10.0)
        ))
    }
    
    fn welcome_card(username: &str, message: &str, on_start: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        NativeComponent(Box::new(
            sized_box(
                flex((
                    label(format!("Welcome back, {}!", username))
                        .text_size(28.0)
                        .weight(xilem::FontWeight::BOLD),
                    FlexSpacer::Fixed(10.0),
                    label(message)
                        .text_size(16.0)
                        .color(map_color(AppColor::TextMuted)),
                    FlexSpacer::Fixed(20.0),
                    button(label("Start Learning").color(Color::WHITE), on_start)
                        .background_color(map_color(AppColor::Primary))
                        .padding(15.0)
                        .corner_radius(10.0),
                ))
                .direction(Axis::Vertical)
                .padding(20.0)
            )
            .background_color(map_color(AppColor::Surface))
            .corner_radius(12.0)
        ))
    }
    
    fn activity_card(title: &str, time: &str, highlighted: bool) -> Self::Output {
        let bg_color = if highlighted {
            Color::from_rgb8(240, 248, 255)
        } else {
            Color::TRANSPARENT
        };
        
        NativeComponent(Box::new(
            flex_row((
                    label(title)
                        .text_size(14.0)
                        .color(map_color(AppColor::Text))
                        .flex(1.0),
                    label(time)
                        .text_size(12.0)
                        .color(map_color(AppColor::TextMuted)),
            ))
            .padding(10.0)
            .background_color(bg_color)
            .corner_radius(5.0)
        ))
    }
    
    fn header_bar(title: &str, on_settings: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        NativeComponent(Box::new(
            flex_row((
                label(title)
                    .text_size(24.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(AppColor::Primary)),
                FlexSpacer::Flex(1.0),
                button("Settings", on_settings)
                    .padding(10.0),
            ))
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .padding(15.0)
            .background_color(map_color(AppColor::Surface))
        ))
    }
    
    fn bottom_nav_bar(_current_screen: Screen, _on_navigate: impl Fn(&mut AppState, Screen) + Send + Sync + 'static) -> Self::Output {
        // Simplified - would need to handle multiple nav items
        NativeComponent(Box::new(
            flex_row(())
                .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
                .padding(10.0)
                .background_color(map_color(AppColor::Surface))
        ))
    }
    
    fn nav_button(label_text: &str, _screen: Screen, is_active: bool, on_click: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        let color = if is_active {
            map_color(AppColor::Primary)
        } else {
            Color::from_rgb8(128, 128, 128)
        };
        
        NativeComponent(Box::new(
            button(label(label_text).color(color), on_click)
                .padding(10.0)
        ))
    }
    
    fn task_presenter(task: &Task, _on_answer: impl Fn(&mut AppState, String) + Send + Sync + 'static) -> Self::Output {
        NativeComponent(Box::new(
            flex((
                label(task.prompt.as_str())
                    .text_size(24.0)
                    .weight(xilem::FontWeight::MEDIUM)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(40.0),
                label("Task Visual Placeholder").color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(40.0),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center)
        ))
    }
    
    fn task_options(_options: Vec<String>, _on_select: impl Fn(&mut AppState, usize) + Send + Sync + 'static) -> Self::Output {
        // Simplified - would need to create buttons for each option
        NativeComponent(Box::new(
            grid((), 2, 2).spacing(15.0)
        ))
    }
    
    fn learning_progress(current: usize, total: usize, accuracy: f64) -> Self::Output {
        NativeComponent(Box::new(
            flex_row((
                label(format!("Task {}/{}", current, total))
                    .text_size(16.0)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(20.0),
                label(format!("Accuracy: {:.0}%", accuracy * 100.0))
                    .text_size(14.0)
                    .color(map_color(AppColor::Success)),
            ))
        ))
    }
    
    fn hint_button(_hint_level: usize, on_hint: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        NativeComponent(Box::new(
            button(label("Hint").color(Color::WHITE), on_hint)
                .padding(10.0)
                .background_color(map_color(AppColor::Warning))
                .corner_radius(5.0)
        ))
    }
    
    fn auth_form(
        username: String,
        password: String,
        on_username: impl Fn(&mut AppState, String) + Send + Sync + 'static,
        on_password: impl Fn(&mut AppState, String) + Send + Sync + 'static,
        on_submit: impl Fn(&mut AppState) + Send + Sync + 'static,
    ) -> Self::Output {
        // Split into nested flex to avoid tuple size limit
        NativeComponent(Box::new(
            flex((
                flex((
                    label("Username").text_size(14.0).color(map_color(AppColor::Text)),
                    FlexSpacer::Fixed(8.0),
                    text_input(username, on_username)
                        .padding(12.0)
                        .corner_radius(8.0),
                    FlexSpacer::Fixed(20.0),
                ))
                .direction(Axis::Vertical),
                flex((
                    label("Password").text_size(14.0).color(map_color(AppColor::Text)),
                    FlexSpacer::Fixed(8.0),
                    text_input(password, on_password)
                        .padding(12.0)
                        .corner_radius(8.0),
                    FlexSpacer::Fixed(30.0),
                ))
                .direction(Axis::Vertical),
                button(label("Login").color(Color::WHITE), on_submit)
                    .background_color(map_color(AppColor::Primary))
                    .padding(15.0)
                    .corner_radius(8.0),
            ))
            .direction(Axis::Vertical)
        ))
    }
    
    fn settings_section(title: &str, children: Vec<Self::Output>) -> Self::Output {
        // For now, just use the first child if any
        let content = if children.is_empty() {
            Box::new(label(title).text_size(18.0).weight(xilem::FontWeight::BOLD)) as Box<AnyWidgetView<AppState>>
        } else {
            Box::new(flex((
                    label(title).text_size(18.0).weight(xilem::FontWeight::BOLD).color(map_color(AppColor::Text)),
                children.into_iter().next().unwrap().build().0,
            ))
            .direction(Axis::Vertical)
            .gap(15.0)) as Box<AnyWidgetView<AppState>>
        };
        
        NativeComponent(Box::new(
            sized_box(content)
                .padding(20.0)
                .background_color(map_color(AppColor::Surface))
                .corner_radius(12.0)
        ))
    }
    
    fn setting_row(label_text: &str, control: Self::Output) -> Self::Output {
        NativeComponent(Box::new(
            flex_row((
                label(label_text).text_size(14.0).flex(1.0),
                control.build().0,
            ))
            .cross_axis_alignment(CrossAxisAlignment::Center)
        ))
    }
    
    fn theme_selector(_current: AppTheme, _on_change: impl Fn(&mut AppState, AppTheme) + Send + Sync + 'static) -> Self::Output {
        // Simplified - would need radio buttons or segmented control
        NativeComponent(Box::new(
            flex_row(()).gap(10.0)
        ))
    }
    
    fn app_scaffold(header: Self::Output, content: Self::Output, footer: Option<Self::Output>) -> Self::Output {
        let scaffold = if let Some(footer) = footer {
            Box::new(
                flex((
                    header.build().0,
                    sized_box(content.build().0).expand(),
                    footer.build().0,
                ))
                .direction(Axis::Vertical)
                .must_fill_major_axis(true)
            ) as Box<AnyWidgetView<AppState>>
        } else {
            Box::new(
                flex((
                    header.build().0,
                    sized_box(content.build().0).expand(),
                ))
                .direction(Axis::Vertical)
                .must_fill_major_axis(true)
            ) as Box<AnyWidgetView<AppState>>
        };
        
        NativeComponent(scaffold)
    }
    
    fn centered_container(max_width: f64, child: Self::Output) -> Self::Output {
        NativeComponent(Box::new(
            sized_box(child.build().0)
                .width(max_width)
        ))
    }
    
    fn stats_grid(stats: Vec<Self::Output>) -> Self::Output {
        // Convert to tuple for up to 4 stats
        let grid = match stats.len() {
            0 => Box::new(label("")) as Box<AnyWidgetView<AppState>>,
            1 => Box::new(stats.into_iter().next().unwrap().build().0) as Box<AnyWidgetView<AppState>>,
            2 => {
                let mut iter = stats.into_iter();
                Box::new(flex_row((
                    iter.next().unwrap().build().0,
                    iter.next().unwrap().build().0,
                )).main_axis_alignment(MainAxisAlignment::SpaceEvenly)) as Box<AnyWidgetView<AppState>>
            },
            3 => {
                let mut iter = stats.into_iter();
                Box::new(flex_row((
                    iter.next().unwrap().build().0,
                    iter.next().unwrap().build().0,
                    iter.next().unwrap().build().0,
                )).main_axis_alignment(MainAxisAlignment::SpaceEvenly)) as Box<AnyWidgetView<AppState>>
            },
            _ => {
                let mut iter = stats.into_iter();
                Box::new(flex_row((
                    iter.next().unwrap().build().0,
                    iter.next().unwrap().build().0,
                    iter.next().unwrap().build().0,
                    iter.next().unwrap().build().0,
                )).main_axis_alignment(MainAxisAlignment::SpaceEvenly)) as Box<AnyWidgetView<AppState>>
            }
        };
        
        NativeComponent(grid)
    }
    
    fn loading_spinner(message: Option<&str>) -> Self::Output {
        let content = if let Some(msg) = message {
            Box::new(
                flex((
                    sized_box(spinner()).width(50.0).height(50.0),
                    label(msg).text_size(16.0).color(map_color(AppColor::TextMuted)),
                ))
                .direction(Axis::Vertical)
                .gap(20.0)
                .cross_axis_alignment(CrossAxisAlignment::Center)
            ) as Box<AnyWidgetView<AppState>>
        } else {
            Box::new(sized_box(spinner()).width(50.0).height(50.0)) as Box<AnyWidgetView<AppState>>
        };
        
        NativeComponent(content)
    }
    
    fn empty_state(icon: &str, title: &str, message: &str, action: Option<Self::Output>) -> Self::Output {
        let content = if let Some(action) = action {
            Box::new(
                flex((
                    label(icon).text_size(48.0),
                    label(title).text_size(20.0).weight(xilem::FontWeight::MEDIUM),
                    label(message).text_size(14.0).color(map_color(AppColor::TextMuted)),
                    action.build().0,
                ))
                .direction(Axis::Vertical)
                .gap(15.0)
                .cross_axis_alignment(CrossAxisAlignment::Center)
            ) as Box<AnyWidgetView<AppState>>
        } else {
            Box::new(
                flex((
                    label(icon).text_size(48.0),
                    label(title).text_size(20.0).weight(xilem::FontWeight::MEDIUM),
                    label(message).text_size(14.0).color(map_color(AppColor::TextMuted)),
                ))
                .direction(Axis::Vertical)
                .gap(15.0)
                .cross_axis_alignment(CrossAxisAlignment::Center)
            ) as Box<AnyWidgetView<AppState>>
        };
        
        NativeComponent(content)
    }
    
    fn error_banner(message: &str, on_dismiss: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        NativeComponent(Box::new(
            flex_row((
                label(message)
                    .color(Color::WHITE)
                    .flex(1.0),
                button("×", on_dismiss)
                    .padding(5.0),
            ))
            .padding(15.0)
            .background_color(map_color(AppColor::Error))
            .corner_radius(8.0)
        ))
    }
    
    fn alphabet_sequence(items: Vec<String>, highlight_index: Option<usize>) -> Self::Output {
        // For now, handle up to 5 items with tuples
        let sequence = match items.len() {
            0 => Box::new(label("")) as Box<AnyWidgetView<AppState>>,
            1 => {
                let is_highlighted = highlight_index == Some(0);
                let bg_color = if is_highlighted {
                    map_color(AppColor::Primary).with_alpha(0.2)
                } else {
                    map_color(AppColor::Surface)
                };
                Box::new(
                    sized_box(label(items[0].clone()).text_size(24.0))
                        .width(60.0).height(60.0)
                        .background_color(bg_color)
                        .corner_radius(8.0)
                ) as Box<AnyWidgetView<AppState>>
            },
            _ => {
                // For simplicity, just show the first few items
                let items_vec: Vec<_> = items.into_iter().take(5).enumerate().collect();
                
                // Create tuple based on number of items
                match items_vec.len() {
                    2 => {
                        let [(i0, item0), (i1, item1)] = items_vec.as_slice() else { unreachable!() };
                        let bg0 = if highlight_index == Some(*i0) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg1 = if highlight_index == Some(*i1) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        Box::new(flex_row((
                            sized_box(label(item0.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg0).corner_radius(8.0),
                            sized_box(label(item1.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg1).corner_radius(8.0),
                        )).gap(10.0)) as Box<AnyWidgetView<AppState>>
                    },
                    3 => {
                        let [(i0, item0), (i1, item1), (i2, item2)] = items_vec.as_slice() else { unreachable!() };
                        let bg0 = if highlight_index == Some(*i0) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg1 = if highlight_index == Some(*i1) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg2 = if highlight_index == Some(*i2) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        Box::new(flex_row((
                            sized_box(label(item0.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg0).corner_radius(8.0),
                            sized_box(label(item1.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg1).corner_radius(8.0),
                            sized_box(label(item2.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg2).corner_radius(8.0),
                        )).gap(10.0)) as Box<AnyWidgetView<AppState>>
                    },
                    4 => {
                        let [(i0, item0), (i1, item1), (i2, item2), (i3, item3)] = items_vec.as_slice() else { unreachable!() };
                        let bg0 = if highlight_index == Some(*i0) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg1 = if highlight_index == Some(*i1) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg2 = if highlight_index == Some(*i2) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg3 = if highlight_index == Some(*i3) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        Box::new(flex_row((
                            sized_box(label(item0.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg0).corner_radius(8.0),
                            sized_box(label(item1.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg1).corner_radius(8.0),
                            sized_box(label(item2.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg2).corner_radius(8.0),
                            sized_box(label(item3.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg3).corner_radius(8.0),
                        )).gap(10.0)) as Box<AnyWidgetView<AppState>>
                    },
                    5 => {
                        let [(i0, item0), (i1, item1), (i2, item2), (i3, item3), (i4, item4)] = items_vec.as_slice() else { unreachable!() };
                        let bg0 = if highlight_index == Some(*i0) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg1 = if highlight_index == Some(*i1) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg2 = if highlight_index == Some(*i2) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg3 = if highlight_index == Some(*i3) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        let bg4 = if highlight_index == Some(*i4) { map_color(AppColor::Primary).with_alpha(0.2) } else { map_color(AppColor::Surface) };
                        Box::new(flex_row((
                            sized_box(label(item0.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg0).corner_radius(8.0),
                            sized_box(label(item1.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg1).corner_radius(8.0),
                            sized_box(label(item2.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg2).corner_radius(8.0),
                            sized_box(label(item3.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg3).corner_radius(8.0),
                            sized_box(label(item4.clone()).text_size(24.0)).width(60.0).height(60.0).background_color(bg4).corner_radius(8.0),
                        )).gap(10.0)) as Box<AnyWidgetView<AppState>>
                    },
                    _ => Box::new(label("Too many items")) as Box<AnyWidgetView<AppState>>,
                }
            }
        };
        
        NativeComponent(sequence)
    }
    
    fn comparison_visual(left: &str, right: &str, show_order: bool) -> Self::Output {
        let arrow = if show_order { "→" } else { "?" };
        
        NativeComponent(Box::new(
            flex_row((
                sized_box(label(left).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background_color(map_color(AppColor::Surface))
                    .border(map_color(AppColor::Primary), 2.0)
                    .corner_radius(12.0),
                label(arrow).text_size(24.0).color(map_color(AppColor::Primary)),
                sized_box(label(right).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background_color(map_color(AppColor::Surface))
                    .border(map_color(AppColor::Primary), 2.0)
                    .corner_radius(12.0),
            ))
            .gap(20.0)
            .main_axis_alignment(MainAxisAlignment::Center)
        ))
    }
    
    fn missing_item_visual(before: &str, after: &str) -> Self::Output {
        NativeComponent(Box::new(
            flex_row((
                sized_box(label(before).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background_color(map_color(AppColor::Surface))
                    .corner_radius(12.0),
                sized_box(label("?").text_size(32.0).color(map_color(AppColor::Primary)))
                    .width(100.0)
                    .height(80.0)
                    .border(map_color(AppColor::Primary), 3.0)
                    .corner_radius(12.0),
                sized_box(label(after).text_size(28.0))
                    .width(100.0)
                    .height(80.0)
                    .background_color(map_color(AppColor::Surface))
                    .corner_radius(12.0),
            ))
            .gap(15.0)
            .main_axis_alignment(MainAxisAlignment::Center)
        ))
    }
    
    fn path_visual(start: &str, end: &str, path: Vec<String>) -> Self::Output {
        // For simplicity, just show start and end
        NativeComponent(Box::new(
            flex_row((
                sized_box(label(start).text_size(24.0))
                    .width(80.0)
                    .height(60.0)
                    .background_color(map_color(AppColor::Primary).with_alpha(0.2))
                    .border(map_color(AppColor::Primary), 2.0)
                    .corner_radius(10.0),
                    label("→").text_size(20.0).color(map_color(AppColor::Primary)),
                    label(if path.is_empty() { "..." } else { "[path]" })
                    .text_size(16.0)
                    .color(map_color(AppColor::TextMuted)),
                label("→").text_size(20.0).color(map_color(AppColor::Primary)),
                sized_box(label(end).text_size(24.0))
                    .width(80.0)
                    .height(60.0)
                    .background_color(map_color(AppColor::Primary).with_alpha(0.2))
                    .border(map_color(AppColor::Primary), 2.0)
                    .corner_radius(10.0),
            ))
            .gap(10.0)
            .main_axis_alignment(MainAxisAlignment::Center)
        ))
    }
    
    fn simple_label(text: String) -> Self::Output {
        NativeComponent(Box::new(
                label(text)
                    .color(map_color(AppColor::Text))
        ))
    }
    
    fn simple_flex_column(items: Vec<Self::Output>) -> Self::Output {
        // Convert Vec of NativeComponents to Vec of AnyWidgetView
        let views: Vec<Box<AnyWidgetView<AppState>>> = items.into_iter().map(|c| c.0).collect();
        NativeComponent(Box::new(
            flex(views).direction(Axis::Vertical).gap(10.0)
        ))
    }
    
    fn simple_flex_row(items: Vec<Self::Output>) -> Self::Output {
        let views: Vec<Box<AnyWidgetView<AppState>>> = items.into_iter().map(|c| c.0).collect();
        NativeComponent(Box::new(
            flex(views).direction(Axis::Horizontal).gap(10.0)
        ))
    }
    
    fn simple_button(text: &str, on_click: impl Fn(&mut AppState) + Send + Sync + 'static) -> Self::Output {
        NativeComponent(Box::new(
            button(label(text), on_click)
                .padding(10.0)
                .corner_radius(5.0)
        ))
    }
    
    // Additional ported components from old app
    fn card<V>(title: &str, content: V) -> Self::Output where V: Component<Output = Self::Output> {
        NativeComponent(Box::new(
            sized_box(
                flex((
                    label(title)
                        .text_size(18.0)
                        .weight(xilem::FontWeight::BOLD)
                        .color(map_color(AppColor::Text)),
                    FlexSpacer::Fixed(10.0),
                    content.build().0,
                ))
                .direction(Axis::Vertical)
                .padding(15.0)
            )
            .background_color(map_color(AppColor::Surface))
            .corner_radius(10.0)
        ))
    }
    
    fn progress_bar(progress: f64, label_text: &str) -> Self::Output {
        let percentage = (progress * 100.0) as u32;
        NativeComponent(Box::new(
            flex((
                label(label_text).text_size(12.0).color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(5.0),
                sized_box(
                    flex(())
                )
                .width(200.0)
                .height(8.0)
                .background_color(Color::from_rgb8(229, 231, 235))
                .corner_radius(4.0),
                FlexSpacer::Fixed(5.0),
                label(format!("{}%", percentage)).text_size(12.0).color(map_color(AppColor::Text)),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Start)
        ))
    }
    
    fn metric_display(label_text: &str, value: &str, color: AppColor) -> Self::Output {
        NativeComponent(Box::new(
            flex((
                label(label_text)
                    .text_size(12.0)
                    .color(map_color(AppColor::TextMuted)),
                FlexSpacer::Fixed(5.0),
                label(value)
                    .text_size(20.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(color)),
            ))
            .direction(Axis::Vertical)
        ))
    }
    
    fn checkbox(checked: bool, label_text: &str, _on_toggle: impl Fn(&mut AppState, bool) + Send + Sync + 'static) -> Self::Output {
        let check_mark = if checked { "☑" } else { "☐" };
        NativeComponent(Box::new(
            flex_row((
                label(check_mark).text_size(16.0).color(map_color(AppColor::Text)),
                label(label_text).text_size(14.0).color(map_color(AppColor::Text)),
            ))
            .gap(8.0)
        ))
    }
    
    fn labeled_input(label_text: &str, value: String, on_change: impl Fn(&mut AppState, String) + Send + Sync + 'static) -> Self::Output {
        NativeComponent(Box::new(
            flex((
                label(label_text).text_size(12.0).color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(5.0),
                text_input(value, on_change)
                    .padding(8.0)
                    .corner_radius(5.0),
            ))
            .direction(Axis::Vertical)
        ))
    }
    
    fn toast_notification(message: &str, is_success: bool) -> Self::Output {
        let bg_color = if is_success {
            map_color(AppColor::Success)
        } else {
            map_color(AppColor::Error)
        };
        
        NativeComponent(Box::new(
            sized_box(
                label(message)
                    .color(Color::WHITE)
                    .text_size(14.0)
            )
            .padding(15.0)
            .background_color(bg_color)
            .corner_radius(8.0)
        ))
    }
    
    fn loading_overlay(message: &str) -> Self::Output {
        NativeComponent(Box::new(
            flex((
                sized_box(spinner()).width(40.0).height(40.0),
                FlexSpacer::Fixed(10.0),
                label(message).text_size(14.0),
            ))
            .direction(Axis::Vertical)
            .cross_axis_alignment(CrossAxisAlignment::Center)
        ))
    }
    
    fn nav_bar(current_screen: &str) -> Self::Output {
        NativeComponent(Box::new(
            sized_box(
                label(format!("📍 {}", current_screen))
                    .text_size(16.0)
            )
            .padding(10.0)
            .background_color(map_color(AppColor::Surface))
        ))
    }
    
    fn error_message(message: Option<String>) -> Self::Output {
        if let Some(msg) = message {
            NativeComponent(Box::new(
                sized_box(
                    label(format!("❌ {}", msg))
                        .color(map_color(AppColor::Error))
                        .text_size(14.0)
                )
                .padding(10.0)
            ))
        } else {
            NativeComponent(Box::new(label("")))
        }
    }
    
    fn success_message(message: Option<String>) -> Self::Output {
        if let Some(msg) = message {
            NativeComponent(Box::new(
                sized_box(
                    label(format!("✅ {}", msg))
                        .color(map_color(AppColor::Success))
                        .text_size(14.0)
                )
                .padding(10.0)
            ))
        } else {
            NativeComponent(Box::new(label("")))
        }
    }
    
    fn highlighted<V>(element_id: &str, content: V, has_highlight: bool, tooltip: Option<String>) -> Self::Output 
        where V: Component<Output = Self::Output> {
        let bg_color = if has_highlight {
            map_color(AppColor::Primary).with_alpha(0.1)
        } else {
            Color::TRANSPARENT
        };
        
        let main_content = Box::new(
            sized_box(content.build().0)
                .background_color(bg_color)
                .border(if has_highlight { map_color(AppColor::Primary) } else { Color::TRANSPARENT }, 2.0)
                .corner_radius(5.0)
        );
        
        if let Some(tip) = tooltip {
            NativeComponent(Box::new(
                flex((
                    main_content,
                    label(format!("💡 {}", tip))
                        .text_size(12.0)
                        .color(map_color(AppColor::Info)),
                ))
                .direction(Axis::Vertical)
                .gap(5.0)
            ))
        } else {
            NativeComponent(main_content)
        }
    }
    
    fn demo_button(element_id: &str, text: &str, has_highlight: bool, tooltip: Option<String>) -> Self::Output {
        let button_view = NativeComponent(Box::new(
            button(label(text), |_| {})
                .padding(10.0)
                .corner_radius(5.0)
        ));
        
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
        NativeComponent(Box::new(
            sized_box(
                flex((
                label(title)
                    .text_size(18.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(AppColor::Text)),
                    FlexSpacer::Fixed(10.0),
                    label(message).text_size(14.0),
                    FlexSpacer::Fixed(20.0),
                    flex_row((
                        button(label("Cancel"), on_cancel)
                            .padding(10.0),
                        button(label("Confirm").color(Color::WHITE), on_confirm)
                            .padding(10.0)
                            .background_color(map_color(AppColor::Primary)),
                    ))
                    .gap(10.0),
                ))
                .direction(Axis::Vertical)
                .padding(20.0)
            )
            .background_color(map_color(AppColor::Surface))
            .corner_radius(12.0)
        ))
    }
    
    fn response_time_histogram(response_times: &[u128]) -> Self::Output {
        let avg = if !response_times.is_empty() {
            response_times.iter().sum::<u128>() / response_times.len() as u128
        } else {
            0
        };
        
        NativeComponent(Box::new(
            flex((
                label("Response Time Histogram")
                    .text_size(16.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(10.0),
                label(format!("📊 Average: {}ms", avg))
                    .text_size(14.0)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(5.0),
                label(format!("{} samples", response_times.len()))
                    .text_size(12.0)
                    .color(map_color(AppColor::TextMuted)),
            ))
            .direction(Axis::Vertical)
        ))
    }
    
    fn learning_curve_display(responses: &[crate::models::Response]) -> Self::Output {
        let correct = responses.iter().filter(|r| r.correct).count();
        let total = responses.len();
        
        NativeComponent(Box::new(
            flex((
                label("Learning Curve")
                    .text_size(16.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(10.0),
                label(format!("📈 {}/{} correct", correct, total))
                    .text_size(14.0)
                    .color(map_color(AppColor::Text)),
            ))
            .direction(Axis::Vertical)
        ))
    }
    
    fn error_analysis_display(responses: &[crate::models::Response]) -> Self::Output {
        let errors = responses.iter().filter(|r| !r.correct).count();
        
        NativeComponent(Box::new(
            flex((
                label("Error Analysis")
                    .text_size(16.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(10.0),
                label(format!("❌ {} errors found", errors))
                    .text_size(14.0)
                    .color(map_color(AppColor::Error)),
            ))
            .direction(Axis::Vertical)
        ))
    }
    
    fn strategy_analysis_display(strategies: Vec<(&str, f64)>) -> Self::Output {
        let strategies_text: Vec<String> = strategies.into_iter()
            .take(3)
            .map(|(name, score)| format!("• {}: {:.0}%", name, score * 100.0))
            .collect();
        
        let content = if strategies_text.is_empty() {
            Box::new(flex((
                label("Strategy Analysis")
                    .text_size(16.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(10.0),
                label("No strategies")
                    .text_size(12.0),
            ))
            .direction(Axis::Vertical)) as Box<AnyWidgetView<AppState>>
        } else if strategies_text.len() == 1 {
            Box::new(flex((
                label("Strategy Analysis")
                    .text_size(16.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(10.0),
                label(strategies_text[0].clone())
                    .text_size(12.0),
            ))
            .direction(Axis::Vertical)) as Box<AnyWidgetView<AppState>>
        } else if strategies_text.len() == 2 {
            Box::new(flex((
                label("Strategy Analysis")
                    .text_size(16.0)
                    .weight(xilem::FontWeight::BOLD),
                FlexSpacer::Fixed(10.0),
                label(strategies_text[0].clone())
                    .text_size(12.0),
                label(strategies_text[1].clone())
                    .text_size(12.0),
            ))
            .direction(Axis::Vertical)
            .gap(5.0)) as Box<AnyWidgetView<AppState>>
        } else {
            Box::new(flex((
                label("Strategy Analysis")
                    .text_size(16.0)
                    .weight(xilem::FontWeight::BOLD),
                FlexSpacer::Fixed(10.0),
                label(strategies_text[0].clone())
                    .text_size(12.0),
                label(strategies_text[1].clone())
                    .text_size(12.0),
                label(strategies_text[2].clone())
                    .text_size(12.0),
            ))
            .direction(Axis::Vertical)
            .gap(5.0)) as Box<AnyWidgetView<AppState>>
        };
        
        NativeComponent(content)
    }
    
    fn domain_card(domain: &str, description: &str, selected: bool) -> Self::Output {
        let bg_color = if selected {
            map_color(AppColor::Primary).with_alpha(0.1)
        } else {
            map_color(AppColor::Surface)
        };
        
        NativeComponent(Box::new(
            sized_box(
                flex((
                label(domain)
                    .text_size(16.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(AppColor::Text)),
                    FlexSpacer::Fixed(5.0),
                label(description)
                    .text_size(12.0)
                    .color(map_color(AppColor::TextMuted)),
                ))
                .direction(Axis::Vertical)
                .padding(15.0)
            )
            .background_color(bg_color)
            .border(if selected { map_color(AppColor::Primary) } else { Color::TRANSPARENT }, 2.0)
            .corner_radius(10.0)
        ))
    }
    
    fn session_info(session_id: &str, status: &str, duration: &str, domain: &str) -> Self::Output {
        NativeComponent(Box::new(
            flex((
                label(format!("Session: {}", session_id))
                    .text_size(14.0)
                    .weight(xilem::FontWeight::BOLD)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(5.0),
                flex_row((
                    label(format!("Status: {}", status)).text_size(12.0).color(map_color(AppColor::Text)),
                    label(format!("Duration: {}", duration)).text_size(12.0).color(map_color(AppColor::Text)),
                    label(format!("Domain: {}", domain)).text_size(12.0).color(map_color(AppColor::Text)),
                ))
                .gap(15.0),
            ))
            .direction(Axis::Vertical)
        ))
    }
    
    fn task_card_display(prompt: &str, hint: Option<&str>) -> Self::Output {
        let content = if let Some(hint_text) = hint {
            Box::new(flex((
                label(prompt)
                    .text_size(18.0)
                    .weight(xilem::FontWeight::MEDIUM)
                    .color(map_color(AppColor::Text)),
                FlexSpacer::Fixed(10.0),
                label(format!("💡 Hint: {}", hint_text))
                    .text_size(12.0)
                    .color(map_color(AppColor::Info)),
            ))
            .direction(Axis::Vertical)) as Box<AnyWidgetView<AppState>>
        } else {
            Box::new(
                label(prompt)
                    .text_size(18.0)
                    .weight(xilem::FontWeight::MEDIUM)
                    .color(map_color(AppColor::Text))
            ) as Box<AnyWidgetView<AppState>>
        };
        
        NativeComponent(Box::new(
            sized_box(content)
                .padding(15.0)
                .background_color(map_color(AppColor::Surface))
                .corner_radius(10.0)
        ))
    }
    
    fn answer_options_display(options: Vec<String>, _on_select: impl Fn(&mut AppState, usize) + Send + Sync + 'static) -> Self::Output {
        // For simplicity, just display the options
        let buttons = options.into_iter().take(4).enumerate().map(|(i, opt)| {
            Box::new(
                button(label(opt), move |_| {
                    // Would call on_select(state, i) here
                })
                .padding(15.0)
                .corner_radius(8.0)
            ) as Box<AnyWidgetView<AppState>>
        }).collect::<Vec<_>>();
        
        match buttons.len() {
            0 => NativeComponent(Box::new(label("No options"))),
            1 => NativeComponent(buttons.into_iter().next().unwrap()),
            2 => {
                let mut iter = buttons.into_iter();
                NativeComponent(Box::new(flex_row((
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                )).gap(10.0)))
            },
            3 => {
                let mut iter = buttons.into_iter();
                NativeComponent(Box::new(flex_row((
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                )).gap(10.0)))
            },
            _ => {
                let mut iter = buttons.into_iter();
                NativeComponent(Box::new(flex_row((
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                    iter.next().unwrap(),
                )).gap(10.0)))
            }
        }
    }
    
    fn performance_chart(accuracy: f64, total_responses: usize, correct: usize, avg_time: f64) -> Self::Output {
        NativeComponent(Box::new(
            sized_box(
                flex((
                    label("Performance")
                        .text_size(16.0)
                        .weight(xilem::FontWeight::BOLD),
                    FlexSpacer::Fixed(10.0),
                    flex_row((
                        Self::metric_display("Accuracy", &format!("{:.0}%", accuracy * 100.0), AppColor::Success).0,
                        Self::metric_display("Total", &total_responses.to_string(), AppColor::Info).0,
                    ))
                    .gap(20.0),
                    FlexSpacer::Fixed(10.0),
                    flex_row((
                        Self::metric_display("Correct", &correct.to_string(), AppColor::Success).0,
                        Self::metric_display("Avg Time", &format!("{:.0}ms", avg_time), AppColor::Info).0,
                    ))
                    .gap(20.0),
                ))
                .direction(Axis::Vertical)
                .padding(15.0)
            )
            .background_color(map_color(AppColor::Surface))
            .corner_radius(10.0)
        ))
    }
}

// Helper function to map app colors to xilem colors
fn map_color(color: AppColor) -> Color {
    match color {
        AppColor::Primary => Color::from_rgb8(59, 130, 246),
        AppColor::Secondary => Color::from_rgb8(147, 51, 234),
        AppColor::Success => Color::from_rgb8(34, 197, 94),
        AppColor::Warning => Color::from_rgb8(251, 146, 60),
        AppColor::Error => Color::from_rgb8(239, 68, 68),
        AppColor::Info => Color::from_rgb8(23, 162, 184),
        AppColor::Surface => Color::from_rgb8(255, 255, 255),
        AppColor::Background => Color::from_rgb8(249, 250, 251),
        AppColor::Text => Color::from_rgb8(17, 24, 39),
        AppColor::TextMuted => Color::from_rgb8(128, 128, 128),
    }
}