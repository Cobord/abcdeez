Of course. Here is an exhaustive developer reference for Xilem, based on the provided source code. This document is designed to be the single source of truth for a code-generation LLM.

### Xilem Developer Reference

---

### **Table of Contents**

1.  [**Introduction**](#1-introduction)
2.  [**Application Lifecycle**](#2-application-lifecycle)
    *   [Simple Single-Window App: `Xilem::new_simple`](#simple-single-window-app-xilemnew_simple)
    *   [Multi-Window & Dynamic Apps: `Xilem::new`](#multi-window--dynamic-apps-xilemnew)
    *   [Window Configuration: `WindowOptions`](#window-configuration-windowoptions)
3.  [**Core Concepts**](#3-core-concepts)
    *   [The `WidgetView` Trait](#the-widgetview-trait)
    *   [State Management](#state-management)
    *   [Actions & Callbacks](#actions--callbacks)
    *   [The `ViewCtx` Context](#the-viewctx-context)
4.  [**Layout Views (Containers)**](#4-layout-views-containers)
    *   [`flex` & `flex_row`](#flex--flex_row)
    *   [`grid`](#grid)
    *   [`zstack`](#zstack)
    *   [`sized_box`](#sized_box)
    *   [`split`](#split)
    *   [`portal`](#portal)
5.  [**Primitive Views (Widgets)**](#5-primitive-views-widgets)
    *   [`label`](#label)
    *   [`button`](#button)
    *   [`text_input`](#text_input)
    *   [`checkbox`](#checkbox)
    *   [`image`](#image)
    *   [`progress_bar`](#progress_bar)
    *   [`spinner`](#spinner)
    *   [`prose`](#prose)
6.  [**Conditional & Dynamic Views**](#6-conditional--dynamic-views)
    *   [Conditional Rendering: `one_of::Either`](#conditional-rendering-one_ofeither)
    *   [Type Erasure: `AnyWidgetView`](#type-erasure-anywidgetview)
    *   [Virtual Scrolling: `virtual_scroll`](#virtual-scrolling-virtual_scroll)
    *   [Indexed Stack: `indexed_stack`](#indexed-stack-indexed_stack)
7.  [**Styling**](#7-styling)
    *   [The `Style` Trait](#the-style-trait)
    *   [Available Style Methods](#available-style-methods)
8.  [**Advanced Views & Concepts**](#8-advanced-views--concepts)
    *   [2D Transformations: `transformed`](#2d-transformations-transformed)
    *   [Asynchronous Operations: `task`](#asynchronous-operations-task)
    *   [Asynchronous Workers: `worker`](#asynchronous-workers-worker)
    *   [Custom Views & `declare_property_tuple!`](#custom-views--declare_property_tuple)
    *   [Internal Details: `Pod`](#internal-details-pod)

---

### 1. Introduction

Xilem is a reactive UI toolkit for Rust. It uses a declarative approach where the UI is a function of the application's state. When the state changes, Xilem efficiently updates the UI to reflect the new state.

The basic model is:
1.  Define your application's `State` as a Rust struct.
2.  Write an `app_logic` function that takes `&mut State` and returns a tree of `View`s.
3.  Use callbacks within your views (e.g., in a `button`) to mutate the `State`.
4.  When the state is mutated, Xilem re-runs the `app_logic` function, diffs the new view tree with the old one, and applies the minimal necessary changes to the UI.

---

### 2. Application Lifecycle

The entry point of every Xilem application is a `Xilem` struct instance, which is then run.

#### Simple Single-Window App: `Xilem::new_simple`

For most applications that use a single, static window, this is the preferred entry point. It automatically handles the application exit when the window is closed.

**Signature:**
```rust
Xilem::new_simple<View>(
    state: State,
    logic: impl FnMut(&mut State) -> View + 'static,
    window_options: WindowOptions<State>,
) -> Self
```

*   `state`: The initial value of your application's state.
*   `logic`: A function (usually a closure) that defines the UI based on the current state.
*   `window_options`: Configuration for the window, such as title and size. See `WindowOptions`.

**Example:**
```rust,no_run
use xilem::{view::{button, flex, label}, EventLoop, WindowOptions, WidgetView, Xilem};

#[derive(Default)]
struct AppState {
    count: i32,
}

fn app_logic(data: &mut AppState) -> impl WidgetView<AppState> {
    flex((
        label(format!("Count: {}", data.count)),
        button("Increment", |data: &mut AppState| data.count += 1),
    ))
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let app = Xilem::new_simple(
        AppState::default(),
        app_logic,
        WindowOptions::new("My App")
    );
    // Use `EventLoop::with_user_event()` for compatibility with async tasks.
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

#### Multi-Window & Dynamic Apps: `Xilem::new`

For applications that require multiple windows, or windows that can be created and destroyed dynamically.

**Signature:**
```rust
Xilem::new(state: State, logic: Logic) -> Self
```

*   `state`: Your application state, which must implement the `AppState` trait.
*   `logic`: A function that takes `&mut State` and returns an `Iterator` of window definitions.

**The `AppState` Trait:**
Your state struct must implement this trait.
```rust
pub trait AppState {
    fn keep_running(&self) -> bool;
}
```
*   `keep_running()`: Determines if the application should exit. This is checked when a window close is requested. If it returns `false`, the entire application will shut down.

**The Logic Function:**
The logic function must return an iterator where each item is a tuple:
`(WindowId, WindowOptions<State>, Box<AnyWidgetView<State>>)`

*   `WindowId`: A unique identifier for the window. Use `WindowId::next()` to generate new ones.
*   `WindowOptions`: Configuration for this specific window.
*   `Box<AnyWidgetView<State>>`: The root view for this window, type-erased.

**Example:**
```rust,no_run
use xilem::{view::{button, label}, AppState, WindowId, WindowOptions, WidgetView, Xilem, EventLoop};
use std::collections::HashMap;

struct MultiWindowState {
    windows: HashMap<WindowId, String>,
}

impl AppState for MultiWindowState {
    // Keep running as long as there is at least one window.
    fn keep_running(&self) -> bool {
        !self.windows.is_empty()
    }
}

fn app_logic(data: &mut MultiWindowState) -> impl Iterator<Item = (WindowId, WindowOptions<MultiWindowState>, Box<dyn WidgetView<MultiWindowState>>)> + '_ {
    data.windows.iter().map(|(id, title)| {
        let window_id = *id;
        let view = button("Close me", move |data: &mut MultiWindowState| {
            data.windows.remove(&window_id);
        });
        let options = WindowOptions::new(title.clone())
            .on_close(move |data: &mut MultiWindowState| {
                data.windows.remove(&window_id);
            });
        (window_id, options, view.boxed())
    })
}

fn main() -> Result<(), winit::error::EventLoopError> {
    let mut initial_state = MultiWindowState { windows: HashMap::new() };
    initial_state.windows.insert(WindowId::next(), "Window 1".to_string());
    initial_state.windows.insert(WindowId::next(), "Window 2".to_string());

    let app = Xilem::new(initial_state, app_logic);
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

#### Window Configuration: `WindowOptions`

Used to configure a window's appearance and behavior.

**Creation:**
`WindowOptions::new(title: impl Into<String>)`

**Methods:**
*   `.on_close(callback: impl Fn(&mut State) + 'static)`: Sets a callback for when the user requests to close the window (e.g., by clicking the 'X' button).
*   `.with_resizable(resizable: bool)`: Default is `true`.
*   `.with_cursor(cursor: impl Into<Cursor>)`: Sets the cursor icon.
*   `.with_min_inner_size(size: S)`: Sets minimum window dimensions.
*   `.with_max_inner_size(size: S)`: Sets maximum window dimensions.
*   `.with_initial_inner_size(size: S)`: Sets the initial size. *Cannot be changed after creation.*
*   `.with_initial_position(position: P)`: Sets the initial position. *Cannot be changed after creation.*
*   `.with_initial_window_icon(icon: Option<Icon>)`: Sets the window icon. *Cannot be changed after creation.*

---

### 3. Core Concepts

#### The `WidgetView` Trait
This is the primary trait that all view-producing functions and structs implement. It's a specialized version of the lower-level `View` trait. Your `app_logic` function must return a type that implements `WidgetView<State>`.

You can use `.boxed()` on any `WidgetView` to type-erase it into a `Box<AnyWidgetView<State, Action>>`.

#### State Management
Xilem follows a "data-down, actions-up" model.
*   **Data Down:** The application `State` is passed down immutably (`&State`) or mutably (`&mut State`) to the `app_logic` function and view-building functions. The views are built based on this state.
*   **Actions Up:** UI elements like `button` take callbacks. These callbacks receive `&mut State` and are the *only* place you should mutate your application state. When a callback mutates the state, Xilem detects the change and triggers a rebuild of the UI.

#### Actions & Callbacks
Callbacks are closures passed to views like `button` or `checkbox`. They define what happens when a user interacts with the UI.

*   A simple callback signature is `|data: &mut MyState| { ... }`.
*   They can also return an `Action`. If your app logic is defined as `fn(&mut State) -> impl WidgetView<State, Action>`, the callback can return a value of type `Action`. This is less common and primarily used for more complex architectures.

#### The `ViewCtx` Context
The `ViewCtx` is a context object passed during the `build` and `rebuild` phases of a view's lifecycle. It tracks the view hierarchy and provides services like access to the async runtime. You will generally not interact with it directly unless you are building custom container views.

---

### 4. Layout Views (Containers)

Layout views arrange their child views. Children are typically provided as a tuple.

#### `flex` & `flex_row`

Arranges children in a single column (`flex`) or row (`flex_row`). `flex_row` is a shorthand for `flex(...).direction(Axis::Horizontal)`.

**Constructors:**
*   `xilem::view::flex(children: impl FlexSequence)`
*   `xilem::view::flex_row(children: impl FlexSequence)`

A tuple of views is a `FlexSequence`, e.g., `(label("one"), button("two", ...))`.

**Configuration Methods:**
*   `.direction(axis: Axis)`: `Axis::Horizontal` or `Axis::Vertical`.
*   `.cross_axis_alignment(axis: CrossAxisAlignment)`: How to align children along the axis perpendicular to the direction. Values: `Start`, `Center`, `End`, `Fill`, `Baseline`.
*   `.main_axis_alignment(axis: MainAxisAlignment)`: How to distribute children along the main direction. Values: `Start`, `Center`, `End`, `SpaceBetween`, `SpaceAround`, `SpaceEvenly`.
*   `.must_fill_major_axis(fill: bool)`: If `true`, the flex container expands to fill available space.
*   `.gap(f64)`: Adds spacing between each child.

**Children (`FlexExt` trait):**
Any `WidgetView` can use methods from the `FlexExt` trait when it's a child of a `flex`.
*   `.flex(params: impl Into<FlexParams>)`: Controls how a child uses space.
    *   `.flex(1.0)`: A flex factor. The child will grow to take up a proportion of the free space.
    *   `.flex(CrossAxisAlignment::Start)`: A specific alignment for this child, overriding the parent's alignment.

**Spacers:**
Special children for adding space.
*   `FlexSpacer::Fixed(f64)`: A fixed-size empty space.
*   `FlexSpacer::Flex(f64)`: A flexible-size empty space (like `.flex(1.0)` on a widget).

**Example:**
```rust
use xilem::view::{flex, label, Axis, FlexExt, FlexSpacer, CrossAxisAlignment};

flex((
    label("Top").flex(CrossAxisAlignment::Start),
    FlexSpacer::Flex(1.0), // Flexible space
    label("Middle"),
    FlexSpacer::Fixed(20.0), // 20px fixed space
    label("Bottom"),
))
.direction(Axis::Vertical)
.cross_axis_alignment(CrossAxisAlignment::Center)
```

#### `grid`

Arranges children in a 2D grid.

**Constructor:**
*   `xilem::view::grid(children: impl GridSequence, width: i32, height: i32)`

**Configuration Methods:**
*   `.spacing(f64)`: Sets both horizontal and vertical spacing between cells.

**Children (`GridExt` trait):**
Children *must* specify their position using methods from the `GridExt` trait.
*   `.grid_pos(x: i32, y: i32)`: Places the child at the given column `x` and row `y`. Assumes a span of 1x1.
*   `.grid_item(params: GridParams)`: For more complex placement. `GridParams::new(x, y, col_span, row_span)` places the widget at `(x,y)` and makes it span multiple cells.

**Example:**
```rust
use xilem::view::{grid, button, label, GridExt};
use masonry::widgets::GridParams;

grid((
    label("Result").grid_item(GridParams::new(0, 0, 4, 1)),
    button("1", |_| {}).grid_pos(0, 1),
    button("2", |_| {}).grid_pos(1, 1),
    button("3", |_| {}).grid_pos(2, 1),
    button("+", |_| {}).grid_pos(3, 1),
), 4, 2)
.spacing(5.0)
```

#### `zstack`

Lays children out on top of each other, from back to front. The size of the `zstack` is the size of its largest child.

**Constructor:**
*   `xilem::view::zstack(children: impl ZStackSequence)`

**Configuration Methods:**
*   `.alignment(alignment: impl Into<UnitPoint>)`: Sets the default alignment for all children. `UnitPoint` specifies a point within the container, e.g., `UnitPoint::CENTER`, `UnitPoint::TOP_LEFT`.

**Children (`ZStackExt` trait):**
*   `.alignment(alignment: impl Into<ChildAlignment>)`: Overrides the parent `zstack`'s alignment for this specific child. `ChildAlignment` can be `ParentAligned` or a specific `UnitPoint`.

**Example:**
```rust
use xilem::view::{zstack, sized_box, label, ZStackExt};
use xilem::Color;
use masonry::properties::types::UnitPoint;

zstack((
    sized_box(())
        .width(100.0)
        .height(100.0)
        .background_color(Color::from_rgb8(0, 0, 200)),
    label("Centered"),
    label("Top Left").alignment(UnitPoint::TOP_LEFT),
))
.alignment(UnitPoint::CENTER)
```

#### `sized_box`

A container that gives its child a specific size, or constraints.

**Constructor:**
*   `xilem::view::sized_box(child: impl WidgetView)`

**Configuration Methods:**
*   `.width(f64)`: Sets a fixed width.
*   `.height(f64)`: Sets a fixed height.
*   `.expand()`: Expands to fill all available space in both dimensions (`width` and `height` become `f64::INFINITY`).
*   `.expand_width()`: Expands to fill available width.
*   `.expand_height()`: Expands to fill available height.

Also implements the `Style` trait for `background_color`, `border`, etc.

**Example:**
```rust
use xilem::view::{sized_box, button};
// A 200x50 button.
sized_box(button("Big Button", |_| {}))
    .width(200.0)
    .height(50.0)
```

#### `split`

A container for two children with a draggable splitter bar between them.

**Constructor:**
*   `xilem::view::split(child1: impl WidgetView, child2: impl WidgetView)`

**Configuration Methods:**
*   `.split_axis(axis: Axis)`: `Axis::Horizontal` (left/right, default) or `Axis::Vertical` (top/bottom).
*   `.split_point(f64)`: The initial position of the splitter, from 0.0 to 1.0. Default is 0.5.
*   `.draggable(bool)`: Whether the splitter can be dragged. Default is `true`.
*   `.min_size(first: f64, second: f64)`: Minimum size for each pane.
*   `.bar_size(f64)`: The visual thickness of the splitter bar.
*   `.solid_bar(bool)`: If `true`, the bar is a solid rectangle. Default is `false` (two lines).

**Example:**
```rust
use xilem::view::{split, label, Axis};

split(
    label("Left Pane"),
    label("Right Pane")
)
.split_axis(Axis::Horizontal)
.split_point(0.25)
```

#### `portal`

A view that makes its child scrollable.

**Constructor:**
*   `xilem::view::portal(child: impl WidgetView)`

**Example:**
```rust
use xilem::view::{portal, flex, label, Axis};
use xilem::WidgetView;

fn long_list() -> impl WidgetView<()> {
    // A very tall flex column
    let children = (0..100).map(|i| label(format!("Item {}", i)));
    flex(children.collect::<Vec<_>>()).direction(Axis::Vertical)
}

// This will create a scrollable view of the long list.
portal(long_list())
```

---

### 5. Primitive Views (Widgets)

These are the fundamental, interactive UI elements.

#### `label`

Displays non-interactive text.

**Constructor:**
*   `xilem::view::label(text: impl Into<ArcStr>)`

**Configuration Methods:**
*   `.text_alignment(TextAlign)`: `Start`, `Middle`, `End`, `Justified`.
*   `.text_size(f32)`: Font size.
*   `.weight(FontWeight)`: e.g., `FontWeight::BOLD`.
*   `.font(impl Into<FontStack>)`: Sets the font family.

Implements `Style` for `color`, etc. `label("...")` is a shortcut for `prose("...").line_break_mode(LineBreaking::Clip)`.

#### `button`

A clickable button.

**Constructors:**
*   `xilem::view::button(label: impl Into<Label>, callback: impl Fn(&mut State) -> Action)`: The primary constructor. The callback is fired on a primary mouse click.
*   `xilem::view::button_any_pointer(...)`: A variant whose callback also receives the `Option<PointerButton>`.

**Configuration Methods:**
*   `.disabled(bool)`: Disables the button if `true`.

Implements `Style` for full customization of background, border, etc., in different states (active, disabled).

#### `text_input`

A field for editable text.

**Constructor:**
*   `xilem::view::text_input(contents: String, on_changed: impl Fn(&mut State, String) -> Action)`

*   `contents`: The current text in the input. This should come from your `State`.
*   `on_changed`: A callback that is fired on every change to the text. You **must** use this callback to update the `contents` in your `State`. Failure to do so will cause the input to reset on the next keystroke.

**Configuration Methods:**
*   `.on_enter(callback: impl Fn(&mut State, String) -> Action)`: A callback fired when the user presses Enter.
*   `.disabled(bool)`: Disables the input.
*   `.text_color(Color)` / `.disabled_text_color(Color)`.
*   `.text_alignment(TextAlign)`.

Implements `Style` for `background_color`, `border`, `padding`, etc.

**Example:**
```rust
use xilem::view::text_input;

struct MyState {
    text: String,
}

text_input(
    data.text.clone(),
    |data: &mut MyState, new_text| {
        data.text = new_text;
    }
)
```

#### `checkbox`

A box that can be checked or unchecked.

**Constructor:**
*   `xilem::view::checkbox(label: impl Into<ArcStr>, checked: bool, callback: impl Fn(&mut State, bool) -> Action)`
*   `label`: The text label next to the checkbox.
*   `checked`: The current state of the checkbox, from your `State`.
*   `callback`: Fired when the state changes. You **must** use it to update your `State`.

**Configuration Methods:**
*   `.disabled(bool)`

Implements `Style`.

#### `image`

Displays a bitmap image.

**Constructor:**
*   `xilem::view::image(image: &vello::peniko::Image)`

**Configuration Methods:**
*   `.fit(ObjectFit)`: Controls how the image scales to fit its container. Values: `Fill` (default), `Contain`, `Cover`, `None`, `ScaleDown`.

#### `progress_bar`

A determinate or indeterminate progress bar.

**Constructor:**
*   `xilem::view::progress_bar(progress: Option<f64>)`
*   `Some(f64)`: A value from 0.0 to 1.0 for a determinate progress bar.
*   `None`: An indeterminate (infinitely animating) progress bar.

#### `spinner`

An infinitely spinning indicator for indeterminate progress.

**Constructor:**
*   `xilem::view::spinner()`

Implements `Style` (primarily for `.color()`).

#### `prose`

Displays immutable, selectable, multi-line text.

**Constructor:**
*   `xilem::view::prose(content: impl Into<ArcStr>)`

**Configuration Methods:**
*   Similar to `label`: `.text_color()`, `.text_alignment()`, `.text_size()`, `.weight()`.
*   `.line_break_mode(LineBreaking)`: Controls text wrapping. `WordWrap` (default), `Clip`, `Overflow`.

---

### 6. Conditional & Dynamic Views

#### Conditional Rendering: `one_of::Either`

`xilem::core::one_of::Either` is the primary tool for conditional rendering (`if/else`). The `app_logic` function must always return the same type. `Either` allows you to return one of two different view types while keeping the outer type the same.

`Either` can be nested for more complex conditions (e.g., `match` statements). For more than 9 branches, use `AnyWidgetView`.

**Example:**
```rust
use xilem::view::{button, label};
use xilem::core::one_of::Either;
use xilem::WidgetView;

fn conditional_view(data: &mut bool) -> impl WidgetView<bool> {
    if *data {
        Either::A(button("Toggle", |data| *data = false))
    } else {
        Either::B(label("Toggled off"))
    }
}
```

#### Type Erasure: `AnyWidgetView`

`AnyWidgetView` is a type-erased `WidgetView`. It is useful when you need to return different concrete view types from a function that cannot be known at compile time, or when you have more conditional branches than `one_of` supports.

It is most commonly used as `Box<dyn AnyWidgetView<State, Action>>`. The `.boxed()` method is a convenient way to create this.

**Example:**
```rust
use xilem::{WidgetView, AnyWidgetView};
use xilem::view::{button, label, checkbox};

fn get_view(name: &str) -> Box<dyn AnyWidgetView<()>> {
    match name {
        "button" => button("a button", |_| {}).boxed(),
        "label" => label("a label").boxed(),
        _ => checkbox("default", false, |_,_| {}).boxed(),
    }
}
```

#### Virtual Scrolling: `virtual_scroll`

A high-performance scrolling container for very long lists. It only builds and renders the children that are currently visible.

**Constructors:**
*   `xilem::view::virtual_scroll(valid_range: Range<i64>, func: F)`
*   `xilem::view::unlimited_virtual_scroll(func: F)`

*   `valid_range`: The total range of items available (e.g., `0..items.len() as i64`).
*   `func`: A "component" function `|data: &mut State, index: i64| -> impl WidgetView`. It is called for each visible item to create its view.

**Important:** The `func` closure is called during the rebuild phase in a special context. Mutating `State` within this function will *not* trigger another top-level application rebuild, to prevent infinite loops.

**Example:**
```rust
use xilem::view::{virtual_scroll, label};
use xilem::WidgetView;

struct AppData {
    items: Vec<String>,
}

fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> {
    let item_count = data.items.len() as i64;
    virtual_scroll(
        0..item_count,
        |data: &mut AppData, index| {
            label(data.items[index as usize].clone())
        }
    )
}
```

#### Indexed Stack: `indexed_stack`

A container that displays only one of its children at a time, based on an index. This is useful for tabs, where you want to keep the state of all tab views alive even when they are not visible.

**Constructor:**
*   `xilem::view::indexed_stack(children: impl IndexedStackSequence)`

**Configuration Methods:**
*   `.active(usize)`: Sets the index of the child to display.

**Example:**
```rust
use xilem::view::{indexed_stack, button, flex, label};

struct AppState {
    active_tab: usize,
}

indexed_stack((
    label("Contents of Tab 1"),
    label("Contents of Tab 2"),
))
.active(data.active_tab)
```

---

### 7. Styling

Styling is applied to views using a builder pattern. Most views implement the `Style` trait.

#### The `Style` Trait
This trait provides a common set of methods for styling. When you call a method like `.background_color()`, it stores that property. When the view is built, these properties are applied to the underlying Masonry widget.

#### Available Style Methods
The following methods are available on most views that implement `Style`.
*   `.background(Background)` / `.background_color(Color)` / `.background_gradient(Gradient)`
*   `.active_background(Background)`: Background when pressed/active.
*   `.disabled_background(Background)`: Background when disabled.
*   `.color(Color)`: Main content color (usually text).
*   `.disabled_color(Color)`: Content color when disabled.
*   `.border(Color, width: f64)`: Sets border color and width.
*   `.border_color(Color)`
*   `.border_width(f64)`
*   `.hovered_border_color(Color)`
*   `.corner_radius(f64)`
*   `.padding(impl Into<Padding>)`: e.g., `.padding(5.0)` or `.padding((10.0, 20.0))`.
*   `.box_shadow(BoxShadow)`

---

### 8. Advanced Views & Concepts

#### 2D Transformations: `transformed`

Applies a 2D affine transformation to a single child view.

**Constructor:**
*   `xilem::view::transformed(child: impl WidgetView)`

**Configuration Methods:**
*   `.translate((x, y))`
*   `.rotate(radians: f64)`
*   `.scale(uniform_factor: f64)`
*   `.scale_non_uniform(x: f64, y: f64)`
*   `.transform(affine: Affine)`: Apply a raw `vello::kurbo::Affine` transform.

The `WidgetView` trait also provides a `.transform(Affine)` method as a shortcut.

**Example:**
```rust
use xilem::view::{transformed, label};

transformed(label("Rotated"))
    .rotate(std::f64::consts::FRAC_PI_4) // Rotate 45 degrees
    .translate((50.0, 0.0))
```

#### Asynchronous Operations: `task`

Runs a one-shot `Future` that can send a single type of message back to the UI thread. The task is cancelled when the view is removed from the tree. The `init_future` function must not capture from its environment.

**Constructor:**
*   `xilem::view::task(init_future, on_event)`
*   `init_future: Fn(MessageProxy<M>) -> Fut`: A function that creates the future. It receives a `MessageProxy` which it can use to send a message of type `M`.
*   `on_event: Fn(&mut State, M) -> Action`: The callback that handles the message `M` received from the future.

**Example: A simple timer**
```rust
use xilem::view::{button, label, task};
use xilem::core::one_of::Either;
use xilem::WidgetView;
use xilem::tokio::time::{sleep, Duration};

struct TimerState {
    timer_done: bool,
}

fn timer_view(data: &mut bool) -> impl WidgetView<bool> {
    if *data {
        Either::A(label("Timer finished!"))
    } else {
        Either::B(
            // The message type is `()`.
            task(
                |proxy| async move {
                    sleep(Duration::from_secs(1)).await;
                    proxy.send(()).unwrap();
                },
                |data, _message| {
                    *data = true;
                }
            )
        )
    }
}
```

#### Asynchronous Workers: `worker`
Runs a long-lived task that can send messages to the UI thread and receive messages from the UI thread. This is for more complex, continuous async communication.

**Constructor:**
*   `xilem::view::worker(init_future, store_sender, on_response)`
*   `init_future: Fn(MessageProxy<M>, UnboundedReceiver<V>) -> Fut`: Creates the worker future. It gets a proxy `M` to send responses and a receiver `V` to get commands.
*   `store_sender: Fn(&mut State, UnboundedSender<V>)`: This callback is run once on build. It must store the `sender` half of the command channel somewhere in the `State` so other parts of the UI can send commands to the worker.
*   `on_response: Fn(&mut State, M) -> Action`: Handles responses of type `M` from the worker.
