Of course. Here is an exhaustive developer reference for Xilem, integrating the provided examples to showcase its capabilities. This document is designed to be the single source of truth for a code-generation LLM.

---

### **Xilem Developer Reference**

This document provides a comprehensive guide to building applications with the Xilem UI toolkit. It covers the application lifecycle, core concepts, all available views and layouts, styling, and advanced patterns, with integrated examples.

### **Table of Contents**

1.  [**Application Lifecycle**](#1-application-lifecycle)
    *   [Simple Single-Window App: `Xilem::new_simple`](#simple-single-window-app-xilemnew_simple)
    *   [Multi-Window & Dynamic Apps: `Xilem::new`](#multi-window--dynamic-apps-xilemnew)
    *   [Window Configuration: `WindowOptions`](#window-configuration-windowoptions)
    *   [External Event Loop Integration](#external-event-loop-integration)
2.  [**Core Concepts & State Management**](#2-core-concepts--state-management)
    *   [The `WidgetView` Trait](#the-widgetview-trait)
    *   [The Reactive Model: State & Callbacks](#the-reactive-model-state--callbacks)
    *   [Component Patterns](#component-patterns)
        *   [Direct Mutation](#direct-mutation)
        *   [State Lensing with `lens`](#state-lensing-with-lens)
        *   [Message Passing with `map_action` and `map_message`](#message-passing-with-map_action-and-map_message)
    *   [Memoization: `memoize` and `frozen`](#memoization-memoize-and-frozen)
    *   [Dependency Injection with `provides` and `with_context`](#dependency-injection-with-provides-and-with_context)
3.  [**Layout Views (Containers)**](#3-layout-views-containers)
    *   [`flex` & `flex_row`](#flex--flex_row)
    *   [`grid`](#grid)
    *   [`zstack`](#zstack)
    *   [`split`](#split)
    *   [`portal` (Scrolling)](#portal-scrolling)
    *   [`sized_box`](#sized_box)
4.  [**Primitive Views (Widgets)**](#4-primitive-views-widgets)
    *   [`label`](#label)
    *   [`button`](#button)
    *   [`text_input`](#text_input)
    *   [`checkbox`](#checkbox)
    *   [`image`](#image)
    *   [`prose`](#prose)
    *   [`progress_bar` & `spinner`](#progress_bar--spinner)
5.  [**Dynamic & Conditional Views**](#5-dynamic--conditional-views)
    *   [Conditional Rendering: `one_of::Either`](#conditional-rendering-one_ofeither)
    *   [Dynamic Lists: `Vec<impl WidgetView>`](#dynamic-lists-vecimpl-widgetview)
    *   [Virtual Scrolling: `virtual_scroll`](#virtual-scrolling-virtual_scroll)
    *   [Tabbed Views: `indexed_stack`](#tabbed-views-indexed_stack)
    *   [Type Erasure: `AnyWidgetView` and `.boxed()`](#type-erasure-anywidgetview-and-boxed)
6.  [**Styling & Transformations**](#6-styling--transformations)
    *   [The `Style` Trait](#the-style-trait)
    *   [2D Transformations: `transformed`](#2d-transformations-transformed)
    *   [Custom Fonts](#custom-fonts)
7.  [**Asynchronous Operations**](#7-asynchronous-operations)
    *   [One-Shot Tasks: `task`](#one-shot-tasks-task)
    *   [Long-Lived Workers: `worker`](#long-lived-workers-worker)

---

### 1. Application Lifecycle

The entry point of a Xilem app is the `Xilem` struct.

#### Simple Single-Window App: `Xilem::new_simple`

This is the standard way to create an application with a single window. It handles the state and event loop setup automatically.

*   **Signature:** `Xilem::new_simple(state: State, logic: impl FnMut(&mut State) -> View, options: WindowOptions<State>)`
*   **`state`**: The initial application state struct.
*   **`logic`**: A function (the "app logic") that takes the current state and returns the UI view tree.
*   **`options`**: Configuration for the window. See `WindowOptions`.

##### **Showcase: Basic Counter App** (from `flex.rs`)
```rust
// The application's state.
struct AppState {
    count: i32,
}

// The function defining the UI based on the state.
fn app_logic(data: &mut i32) -> impl WidgetView<i32> {
    flex_row((
        // A button that mutates the state when clicked.
        button("-", |data| *data -= 1),
        label(format!("count: {}", data)),
        button("+", |data| *data += 1),
    ))
}

fn main() -> Result<(), EventLoopError> {
    // Create the app instance.
    let app = Xilem::new_simple(0, app_logic, WindowOptions::new("My App"));
    // Run the app. `with_user_event()` is required for async tasks.
    app.run_in(EventLoop::with_user_event())?;
    Ok(())
}
```

#### Multi-Window & Dynamic Apps: `Xilem::new`

For applications requiring multiple, dynamically managed windows.

*   **Signature:** `Xilem::new(state: State, logic: Logic)`
*   **`state`**: The app state, which must implement the `AppState` trait.
*   **`logic`**: A function returning an `Iterator` of window definitions: `(WindowId, WindowOptions<State>, Box<AnyWidgetView<State>>)`

The `AppState` trait requires `fn keep_running(&self) -> bool;`, which determines if the application should exit.

##### **Showcase: Multi-Window Counter Manager** (from `multiple_windows.rs`)
```rust
struct State {
    counters: HashMap<WindowId, Counter>,
    running: bool,
    main_window_id: WindowId,
    // ... other fields
}

impl AppState for State {
    fn keep_running(&self) -> bool {
        self.running
    }
}

// App logic returns an iterator of window definitions.
fn app_logic(
    state: &mut State,
) -> impl Iterator<Item = (WindowId, WindowOptions<State>, Box<AnyWidgetView<State>>)> {
    // The main control window.
    let main_window = (
        state.main_window_id,
        WindowOptions::new("Multiple windows").on_close(|state: &mut State| {
            state.running = false; // Set running to false to exit the app.
        }),
        flex((
            // ... UI to add new counter windows
            button("Add", |state: &mut State| {
                state.counters.insert(WindowId::next(), Counter { name: /*...*/, value: 0 });
            }),
        )).boxed(), // The root view must be boxed.
    );

    // An iterator for all the dynamic counter windows.
    let counter_windows = state.counters.iter().map(|(window_id, counter)| {
        (
            *window_id,
            WindowOptions::new(&counter.name).on_close(move |state: &mut State| {
                state.counters.remove(&window_id); // Close only this window.
            }),
            flex((
                label(format!("count: {}", counter.value)),
                button("+", move |state: &mut State| {
                    state.counters.get_mut(&window_id).unwrap().value += 1;
                }),
            )).boxed(),
        )
    });

    std::iter::once(main_window).chain(counter_windows)
}
```

#### Window Configuration: `WindowOptions`

Configures a window's appearance and behavior.

*   **Creation:** `WindowOptions::new(title: impl Into<String>)`
*   **Methods:**
    *   `.on_close(callback)`: Handle the window close request.
    *   `.with_resizable(bool)`
    *   `.with_cursor(Cursor)`
    *   `.with_min_inner_size(Size)`, `.with_max_inner_size(Size)`
    *   `.with_initial_inner_size(Size)`: *Cannot be changed after creation.*
    *   `.with_initial_position(Position)`: *Cannot be changed after creation.*
    *   `.with_initial_window_icon(Option<Icon>)`: *Cannot be changed after creation.*

#### External Event Loop Integration

For advanced use cases, you can integrate Xilem into an existing Winit event loop.

*   **Method:** `xilem.into_driver_and_windows(...)`
*   This splits the `Xilem` app into a `driver` (the app logic) and the initial `windows`. You then manually drive the application by passing Winit events to `masonry_winit::app::MasonryState`.

##### **Showcase:** (from `external_event_loop.rs`)
```rust
// An application not managed by Xilem.
struct ExternalApp {
    masonry_state: masonry_winit::app::MasonryState<'static>,
    app_driver: Box<dyn AppDriver>,
}

// Manually implement winit's ApplicationHandler.
impl ApplicationHandler<MasonryUserEvent> for ExternalApp {
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Forward events to Masonry.
        self.masonry_state.handle_window_event(
            event_loop,
            window_id,
            event,
            self.app_driver.as_mut(),
        );
    }
    // ... other event handlers ...
}

fn main() -> Result<(), EventLoopError> {
    let xilem = Xilem::new_simple(/* ... */);
    let event_loop = EventLoop::with_user_event().build().unwrap();
    let proxy = event_loop.create_proxy();

    // Deconstruct the Xilem app.
    let (driver, windows) =
        xilem.into_driver_and_windows(move |event| proxy.send_event(event).map_err(|err| err.0));
    
    // Create Masonry's state manager.
    let masonry_state = masonry_winit::app::MasonryState::new(proxy, windows, /*...*/);

    let mut app = ExternalApp {
        masonry_state,
        app_driver: Box::new(driver),
    };
    // Run the external app loop.
    event_loop.run_app(&mut app)
}
```

---

### 2. Core Concepts & State Management

#### The `WidgetView` Trait

The fundamental trait for all UI components. Your `app_logic` function must return a type implementing `WidgetView<YourState>`.

#### The Reactive Model: State & Callbacks

Xilem operates on a simple principle: **the UI is a function of the state**.

1.  **State**: A single Rust struct (`struct AppState { ... }`) holds all data for your application.
2.  **View**: The `app_logic` function reads this state and returns a tree of views.
3.  **Action**: User interactions (like button clicks) trigger callbacks.
4.  **Update**: These callbacks are the *only* place where you should mutate the state (`|data: &mut AppState| data.count += 1`).
5.  **Re-render**: After a mutation, Xilem re-runs `app_logic`, diffs the new view tree against the old one, and applies the minimal necessary changes to the screen.

#### Component Patterns

Xilem offers several patterns for breaking down your UI and state into manageable components.

##### Direct Mutation

The simplest pattern. A component is just a function that returns a view and whose callbacks mutate the state directly.

##### **Showcase: A Simple To-Do List** (from `to_do_mvc.rs`)
```rust
struct Task {
    description: String,
    done: bool,
}

struct TaskList {
    tasks: Vec<Task>,
    // ...
}

fn app_logic(task_list: &mut TaskList) -> impl WidgetView<TaskList> {
    let tasks_view = task_list
        .tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            flex_row((
                checkbox(
                    task.description.clone(),
                    task.done,
                    // The callback receives the *entire* app state
                    // and mutates the specific task.
                    move |data: &mut TaskList, checked| {
                        data.tasks[i].done = checked;
                    },
                ),
                button("Delete", move |data: &mut TaskList| {
                    data.tasks.remove(i);
                }),
            ))
        })
        .collect::<Vec<_>>();

    flex((/* ... */, tasks_view))
}
```

##### State Lensing with `lens`

`lens` allows you to create a component that operates on a *subset* of the main application state.

*   **Signature:** `lens(component: impl Fn(&mut S) -> V, lense_fn: impl Fn(&mut T) -> &mut S)`
*   **`component`**: A view function that works with the smaller state `S`.
*   **`lense_fn`**: A closure that provides a mutable reference to the smaller state `S` from the larger state `T`.

##### **Showcase: A Modular Counter** (from `components.rs`)
```rust
struct AppState {
    modularized_count: i32,
    global_count: i32,
}

// This component only knows about `i32`, not `AppState`.
fn modular_counter(count: &mut i32) -> impl WidgetView<i32> {
    flex((
        label(format!("modularized count: {}", count)),
        button("+", |count: &mut i32| *count += 1),
    ))
}

fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> {
    flex_row((
        // Use `lens` to adapt `modular_counter` to work on `AppState`.
        lens(
            modular_counter, // The component.
            |state: &mut AppState| &mut state.modularized_count, // The "lens" function.
        ),
        // ... other global state UI ...
    ))
}
```

##### Message Passing with `map_action` and `map_message`

This pattern, inspired by The Elm Architecture, allows a child component to send a "message" (an enum) to its parent instead of mutating the state directly. The parent then handles the message and performs the mutation.

*   **`map_action(child_view, handler)`**: The child view returns a message of type `M`. The handler `|state: &mut State, msg: M|` processes it.
*   **`map_message(child_view, handler)`**: A more powerful version where the handler receives the full `MessageResult<M>` and can decide whether to request a rebuild, do nothing, or perform an action.

##### **Showcase: Elm-style Counter** (from `elm.rs`)
```rust
// The message (or action) the child component can send.
enum CountMessage {
    Increment,
    Decrement,
}

// The child component returns a `CountMessage` instead of mutating state.
fn elm_counter(count: i32) -> impl WidgetView<i32, CountMessage> {
    flex((
        label(format!("elm count: {count}")),
        button("+", |_| CountMessage::Increment),
        button("-", |_| CountMessage::Decrement),
    ))
}

fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> {
    // `map_action` catches the message and mutates the state.
    map_action(
        elm_counter(state.map_action_count),
        |state: &mut AppState, message| match message {
            CountMessage::Increment => state.map_action_count += 1,
            CountMessage::Decrement => state.map_action_count -= 1,
        },
    )
}
```

#### Memoization: `memoize` and `frozen`

To optimize performance, you can prevent parts of the view tree from being re-computed and re-built if their underlying data hasn't changed.

*   **`memoize(key, view_fn)`**: Re-runs `view_fn` and rebuilds the view only when the `key` changes. The key must be `PartialEq + Clone`.
*   **`frozen(view_fn)`**: A special case of `memoize` where the view is computed only once, ever. Ideal for static UI elements.

##### **Showcase: Memoized Buttons** (from `memoization.rs`)
```rust
// `memoize` depends on a key (`state.count`). The button is only rebuilt
// when the count changes.
fn decrease_button(state: &AppState) -> impl WidgetView<AppState> {
    memoize(state.count, |count| {
        button(
            format!("decrease the count: {}", count),
            |data: &mut AppState| data.count -= 1,
        )
    })
}

// `frozen` is for views that are completely static.
fn reset_button() -> impl WidgetView<AppState> {
    frozen(|| button("reset", |data: &mut AppState| data.count = 0))
}
```

#### Dependency Injection with `provides` and `with_context`

For passing data down the view tree without "prop drilling" (passing it through every intermediate component).

*   **`provides(data_fn, child_view)`**: Makes a piece of data (a `Resource`) available to all descendant views. `data_fn` is a closure `|state: &mut State| -> MyResource` that creates the resource.
*   **`with_context(view_fn)`**: A descendant view that can access the provided resource. `view_fn` is a closure `|context: &mut MyResource, state: &mut State| -> impl WidgetView`.

##### **Showcase:** (from `mason.rs`)
```rust
// A resource must implement the `Resource` trait.
#[derive(Debug)]
struct SomeContext(u32);
impl Resource for SomeContext {}

// A component that needs the resource.
fn env_using() -> impl WidgetView<AppData> {
    // `with_context` gets access to the provided resource.
    with_context(|context: &mut SomeContext, _| {
        button(format!("Context: {}", context.0), |_: &mut AppData| {})
    })
}

fn app_logic(data: &mut AppData) -> impl WidgetView<AppData> {
    // The parent provides the resource to its children.
    provides(
        |_: &mut AppData| SomeContext(120),
        flex((
            env_using(), // This child can now access `SomeContext`.
            // ...
        ))
    )
}
```

---

### 3. Layout Views (Containers)

Layout views arrange their children. Children are passed as a tuple, e.g., `flex((child1, child2))`.

#### `flex` & `flex_row`

Arranges children in a single column (`flex`) or row (`flex_row`).

*   **Constructors:** `flex(children)`, `flex_row(children)`
*   **Config Methods:**
    *   `.direction(Axis)`: `Vertical` or `Horizontal`.
    *   `.cross_axis_alignment(CrossAxisAlignment)`: Align children across the main axis (`Start`, `Center`, `End`, `Fill`).
    *   `.main_axis_alignment(MainAxisAlignment)`: Distribute children along the main axis (`Start`, `Center`, `End`, `SpaceBetween`, etc.).
    *   `.gap(f64)`: Space between children.
*   **Children (`FlexExt` trait):**
    *   `.flex(f64)`: A child with a flex factor grows to fill proportional space.
*   **Spacers:**
    *   `FlexSpacer::Fixed(f64)`: Fixed-size space.
    *   `FlexSpacer::Flex(f64)`: Flexible space.

##### **Showcase:** (from `flex.rs`)
```rust
fn app_logic(data: &mut i32) -> impl WidgetView<i32> {
    flex_row((
        FlexSpacer::Fixed(30.0),
        button("-", |data| *data -= 1),
        FlexSpacer::Flex(1.0), // Takes up 1 part of flexible space.
        label(format!("count: {}", data)).flex(5.0), // Takes up 5 parts.
        FlexSpacer::Flex(1.0),
        button("+", |data| *data += 1),
        FlexSpacer::Fixed(30.0),
    ))
    .cross_axis_alignment(CrossAxisAlignment::Center)
    .main_axis_alignment(MainAxisAlignment::Center)
}
```

#### `grid`

Arranges children in a 2D grid.

*   **Constructor:** `grid(children, width: i32, height: i32)`
*   **Config Methods:** `.spacing(f64)`
*   **Children (`GridExt` trait):** Children *must* specify their position.
    *   `.grid_pos(col: i32, row: i32)`
    *   `.grid_item(GridParams::new(col, row, col_span, row_span))`

##### **Showcase: Calculator Layout** (from `calc.rs`)
```rust
grid((
    // Display spans all 4 columns of the first row.
    display.grid_item(GridParams::new(0, 0, 4, 1)),
    
    // Top row of buttons
    button("CE", ...).grid_pos(0, 1),
    button("C", ...).grid_pos(1, 1),
    button("DEL", ...).grid_pos(2, 1),
    button("÷", ...).grid_pos(3, 1),
    
    // ... more rows ...
), 4, 6) // 4 columns, 6 rows
.spacing(2.0)
```

#### `zstack`

Lays children on top of each other, from back to front.

*   **Constructor:** `zstack(children)`
*   **Config Methods:** `.alignment(UnitPoint)` sets default alignment (e.g., `UnitPoint::CENTER`).
*   **Children (`ZStackExt` trait):** `.alignment(UnitPoint)` overrides the parent alignment for a specific child.

##### **Showcase: Image with Attribution Overlay** (from `http_cats.rs`)
```rust
let attribution = sized_box(/* ... prose ... */)
    .padding(4.)
    .background_color(/* semi-transparent black */);

// The `zstack` layers the attribution text on top of the image.
zstack((
    image(image_data),
    // Align the attribution to the top-right corner of the zstack.
    attribution.alignment(UnitPoint::TOP_RIGHT),
))
```

#### `split`

A container with two panes and a draggable splitter.

*   **Constructor:** `split(child1, child2)`
*   **Config Methods:**
    *   `.split_axis(Axis)`: `Horizontal` (default) or `Vertical`.
    *   `.split_point(f64)`: Initial position of splitter (0.0 to 1.0).
    *   `.draggable(bool)`

##### **Showcase:** (from `http_cats.rs`)
```rust
// A vertically split view with a list on the left and details on the right.
split(
    list_of_status_codes,
    details_view
)
.split_point(0.4) // Left pane takes 40% of the width.
.split_axis(Axis::Horizontal)
```

#### `portal` (Scrolling)

Makes its child content scrollable.

*   **Constructor:** `portal(child)`

##### **Showcase:** (from `variable_clock.rs`)
```rust
// The flex container is very tall. `portal` makes it scrollable.
portal(
    flex(
        TIMEZONES.iter().map(|it| it.view(data)).collect::<Vec<_>>(),
    )
).flex(1.) // Make the portal take up remaining space.
```

#### `sized_box`

Constrains its child's size.

*   **Constructor:** `sized_box(child)`
*   **Config Methods:** `.width(f64)`, `.height(f64)`, `.expand()`, `.expand_width()`, `.expand_height()`.
*   Implements `Style` for background, border, etc.

##### **Showcase:** (from `calc.rs`)
```rust
// Creates a button that expands to fill its grid cell.
fn expanded_button(
    text: impl Into<Label>,
    callback: impl Fn(&mut Calculator),
) -> impl WidgetView<Calculator> {
    sized_box(
        button(text, callback)
            .background_color(/*...*/)
    )
    .expand() // The key method call.
}
```

---

### 4. Primitive Views (Widgets)

#### `label`

Displays non-interactive text.

*   **Constructor:** `label(text)`
*   **Config Methods:** `.text_size(f32)`, `.weight(FontWeight)`, `.text_alignment(TextAlign)`, `.color(Color)`.

#### `button`

A clickable button. The `button_any_pointer` variant also receives which mouse button was pressed.

*   **Constructor:** `button(label, callback)`
*   **Config Methods:** `.disabled(bool)`. Implements `Style`.

#### `text_input`

An editable text field. It's a "controlled component."

*   **Constructor:** `text_input(contents, on_changed)`
*   `contents: String`: The current value from your `State`.
*   `on_changed: |&mut State, String|`: Callback to update `contents` in your `State` on every keystroke. **This is mandatory for the input to function correctly.**
*   **Config Methods:** `.on_enter(|&mut State, String|)`, `.insert_newline(InsertNewline)`.

#### `checkbox`

A stateful checkbox. Also a "controlled component."

*   **Constructor:** `checkbox(label, checked, on_changed)`
*   `checked: bool`: The current value from your `State`.
*   `on_changed: |&mut State, bool|`: Callback to update `checked` in your `State`. **Mandatory.**

#### `image`

Displays a bitmap image.

*   **Constructor:** `image(&vello::peniko::Image)`
*   **Config Methods:** `.fit(ObjectFit)`: (`Fill`, `Contain`, `Cover`, `None`, `ScaleDown`).

#### `prose`

Displays selectable, multi-line, read-only text.

*   **Constructor:** `prose(text)`
*   **Config Methods:** `.text_alignment(TextAlign)`, `.line_break_mode(LineBreaking)`.

#### `progress_bar` & `spinner`

For showing progress.

*   `progress_bar(Option<f64>)`: `Some(0.0..=1.0)` for determinate, `None` for indeterminate.
*   `spinner()`: An infinitely spinning indeterminate indicator.

---

### 5. Dynamic & Conditional Views

#### Conditional Rendering: `one_of::Either`

`xilem::core::one_of::Either` is used for compile-time conditional views (`if/else`). The `app_logic` function must always return a single, concrete type. `Either` unifies two different view types into one.

*   `Either::A(view1)` / `Either::B(view2)`
*   For more than two branches, `OneOf` types are available (e.g., `OneOf3`). Use `OneOfN::A(...)` for the first branch to establish the type, then subsequent branches can use `OneOf::B(...)`, `OneOf::C(...)`.

##### **Showcase: A State Machine** (from `state_machine.rs`)
```rust
fn state_machine(app_data: &mut StateMachine) -> impl WidgetView<StateMachine> {
    match app_data.state {
        // First branch must specify the number of variants.
        IsEven::Initial | IsEven::Even => OneOf3::A(flex((
            // ... buttons ...
        ))),
        // Subsequent branches use the generic `OneOf`.
        IsEven::Odd => OneOf::B(flex((
            // ... buttons ...
        ))),
        // Multiple logical branches can map to the same view type and `OneOf` variant.
        IsEven::Halt => OneOf::C(label("Failure!")),
        IsEven::Success => OneOf::C(label("Success!")),
    }
}
```

#### Dynamic Lists: `Vec<impl WidgetView>`

To display a list of items of a variable length, create a `Vec` of views and include it in a layout tuple.

##### **Showcase:** (from `lists.rs`)
```rust
fn app_logic(state: &mut AppState) -> impl WidgetView<AppState> {
    // Create a vector of views based on the state.
    let list = (0..state.count)
        .map(|n| prose(format!("item #{}", n)))
        .collect::<Vec<_>>();

    flex((
        // The vector can be part of a tuple with other views.
        button("more", |appstate: &mut AppState| appstate.count += 1),
        list,
    ))
    .direction(Axis::Vertical)
}
```

#### Virtual Scrolling: `virtual_scroll`

A high-performance container for very long lists, which only renders visible items.

*   **Constructors:** `virtual_scroll(valid_range: Range<i64>, func)`, `unlimited_virtual_scroll(func)`
*   `func: |&mut State, index: i64| -> impl WidgetView`: A component function called to create the view for each visible item.

##### **Showcase: A List of Downloadable Cat Pictures** (from `virtual_cats.rs`)
```rust
fn view(&mut self) -> impl WidgetView<Self> {
    // `virtual_scroll` is given the total range of items and a function
    // to create the view for any given index.
    virtual_scroll(
        0..self.statuses.len() as i64,
        Self::virtual_item,
    )
}

// The item-creation function. It's called on-demand for visible items.
// Here, it also forks an async task to download the image if it's not present.
fn virtual_item(&mut self, idx: i64) -> impl WidgetView<Self> {
    let index = idx as usize;
    let item = &mut self.statuses[index];
    
    let image_view = match &item.image {
        ImageState::Available(image) => Either::A(image(image)),
        _ => Either::B(spinner()), // Show a spinner while loading.
    };

    let download_task = if matches!(&item.image, ImageState::Pending) {
        // Create an async task to download the image.
        Some(task_raw(/* ... */))
    } else {
        None
    };
    
    // `fork` combines a view with an optional async task.
    fork(flex((prose(item.message.clone()), image_view)), download_task)
}
```

#### Tabbed Views: `indexed_stack`

Displays only one of its child views at a time, selected by an index. Keeps all children's state alive.

*   **Constructor:** `indexed_stack(children)`
*   **Config Methods:** `.active(usize)`

##### **Showcase: A Widget Gallery with Tabs** (from `widgets.rs`)
```rust
enum GalleryTab {
    Progress = 0,
    Checkbox,
}
struct WidgetGallery {
    tab: GalleryTab,
    // ... other state
}

fn app_logic(data: &mut WidgetGallery) -> impl WidgetView<WidgetGallery> {
    flex((
        flex_row(( // The tab buttons
            button("Progress", |data| data.tab = GalleryTab::Progress),
            button("Checkbox", |data| data.tab = GalleryTab::Checkbox),
        )),
        // The indexed_stack displays one of the views based on the active tab index.
        indexed_stack((
            // View for tab 0
            border_box(progress_bar_view(...)),
            // View for tab 1
            border_box(checkbox_view(...)),
        ))
        .active(data.tab as usize), // Set the active child.
    ))
}
```

#### Type Erasure: `AnyWidgetView` and `.boxed()`

Used when a function must return different, non-unifiable view types. Calling `.boxed()` on any `WidgetView` erases its concrete type into a `Box<dyn AnyWidgetView<...>>`. This is essential for dynamic UIs like the multi-window example.

---

### 6. Styling & Transformations

#### The `Style` Trait

Most views implement the `Style` trait, providing a builder-style API for visual properties.

*   **Methods:** `.background_color(Color)`, `.border(Color, width)`, `.corner_radius(f64)`, `.padding(f64)`, etc.

##### **Showcase:** (from `calc.rs`)
```rust
// A styled button for a calculator.
fn expanded_button(
    text: impl Into<Label>,
    callback: impl Fn(&mut Calculator),
) -> impl WidgetView<Calculator> {
    const BLUE: Color = Color::from_rgb8(0x00, 0x8d, 0xdd);
    sized_box(
        button(text, callback)
            .background_color(BLUE)
            .corner_radius(10.)
            .border_color(Color::TRANSPARENT)
            .hovered_border_color(Color::WHITE),
    )
    .expand()
}
```

#### 2D Transformations: `transformed`

Applies 2D affine transformations to its child.

*   **Constructor:** `transformed(child)`
*   **Config Methods:** `.rotate(radians)`, `.scale(factor)`, `.translate((x, y))`. Transformations are applied in the order they are called.

##### **Showcase: A Mini-game to Reset Transformations** (from `transforms.rs`)
```rust
fn view(&mut self) -> impl WidgetView<Self> {
    // ...
    // The order matters: scale, then rotate, then translate.
    let transformed_status = transformed(status)
        .scale(self.scale)
        .rotate(self.rotation)
        .translate(self.translation);
    
    grid((
        // ... controls to change rotation, scale, translation ...
        transformed_status.grid_pos(1, 1),
    ), 3, 3)
}
```

#### Custom Fonts

Custom fonts can be loaded when the application starts.

*   **Method:** `Xilem::... .with_font(Blob<u8>)`
*   Once loaded, refer to the font by its family name as a string in `.font("MyFontFamilyName")`.

##### **Showcase: A Clock with a Variable Font** (from `variable_clock.rs`)
```rust
const ROBOTO_FLEX: &[u8] = include_bytes!(/* ... */);

fn main() {
    // ...
    // Load the font when creating the app.
    let app = Xilem::new_simple(/*...*/)
        .with_font(Blob::new(Arc::new(ROBOTO_FLEX)));
    app.run_in(/*...*/)
}

// A component that uses the loaded font.
fn clock_display(...) -> impl WidgetView<Clocks> {
    variable_label("12:34:56")
        // Refer to the font by its name.
        .font("Roboto Flex")
        .target_weight(data.weight, 400.)
}
```

---

### 7. Asynchronous Operations

#### One-Shot Tasks: `task`

Runs a `Future` that can send a message back to the UI thread. The task is cancelled if its view is removed from the tree.

*   **Constructor:** `task(init_future, on_event)`
*   `init_future: |MessageProxy<M>| -> Fut`: Creates the future. The `proxy` is used to send a message of type `M` back.
*   `on_event: |&mut State, M|`: Handles the message from the future.

##### **Showcase: A Stopwatch Timer** (from `stopwatch.rs`)
```rust
fn app_logic(data: &mut Stopwatch) -> impl WidgetView<Stopwatch> {
    // `fork` combines a primary view with an optional secondary view (like a task).
    fork(
        flex((
            // ... display and buttons ...
        )),
        // The task is only present in the view tree if `data.active` is true.
        data.active.then(|| {
            // This task ticks every 50ms and sends a `()` message.
            task(
                |proxy| async move {
                    let mut interval = time::interval(Duration::from_millis(50));
                    loop {
                        interval.tick().await;
                        proxy.message(()).unwrap();
                    }
                },
                // The handler updates the displayed time.
                |data: &mut Stopwatch, ()| {
                    data.update_display();
                },
            )
        }),
    )
}
```

#### Long-Lived Workers: `worker`

For continuous, two-way async communication. The worker runs for the lifetime of the view.

*   **Constructor:** `worker(init_future, store_sender, on_response)`
*   `init_future: |MessageProxy<M>, UnboundedReceiver<V>| -> Fut`: The worker future receives commands of type `V` and sends back responses of type `M`.
*   `store_sender: |&mut State, UnboundedSender<V>|`: A one-time setup callback to store the command sender in your `State`.
*   `on_response: |&mut State, M|`: Handles responses from the worker.

##### **Showcase: An Image Download Pool** (from `http_cats.rs`)
```rust
struct HttpCats {
    // ...
    download_sender: Option<UnboundedSender<u32>>,
}

fn view(&mut self) -> impl WidgetView<Self> {
    fork(
        // ... main UI ...
        split(...),
        // The worker runs in the background.
        worker(
            // The worker future. It receives a status code to download...
            |proxy, mut rx| async move {
                while let Some(code) = rx.recv().await {
                    let proxy = proxy.clone();
                    tokio::spawn(async move {
                        let image = image_from_url(&format!("https://http.cat/{}", code)).await;
                        // ...and sends the downloaded image back as a response.
                        proxy.message((code, image.unwrap())).unwrap();
                    });
                }
            },
            // `store_sender` saves the command sender into our state.
            |state: &mut Self, sender| {
                state.download_sender = Some(sender);
            },
            // `on_response` handles the downloaded image and updates the state.
            |state: &mut Self, (code, image): (u32, Image)| {
                if let Some(status) = state.statuses.iter_mut().find(|it| it.code == code) {
                    status.image = ImageState::Available(image);
                }
            },
        ),
    )
}

// A button somewhere else in the UI can now use the sender to command the worker.
fn list_item_view(&mut self) -> impl WidgetView<HttpCats> {
    button("Select", move |state: &mut HttpCats| {
        // Send a command to the worker to download the image for this code.
        state.download_sender.as_ref().unwrap().send(code).unwrap();
        // ...
    })
}
```