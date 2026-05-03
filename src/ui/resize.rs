//! Wraps a window's content with invisible resize handles around its edges
//! and corners. Each handle starts a native winit resize gesture via
//! `window::drag_resize` so the window grows/shrinks at OS framerate without
//! the iced event loop in the middle.
//!
//! The ring is laid on top of the content with `Stack`, so it does not steal
//! any layout space — clicks at the edges hit `mouse_area`, clicks anywhere
//! else fall through to the underlying widgets.

use iced::widget::{column, mouse_area, row, stack, Space};
use iced::window::{self, Direction};
use iced::{mouse, Element, Length};

use crate::app::Message;

const THICKNESS: f32 = 5.0;

pub fn wrap<'a>(
    content: Element<'a, Message>,
    window_id: window::Id,
    enabled: bool,
) -> Element<'a, Message> {
    if !enabled {
        return content;
    }

    stack![content, ring(window_id)].into()
}

fn ring(window_id: window::Id) -> Element<'static, Message> {
    let nw = corner(window_id, Direction::NorthWest);
    let n = edge(window_id, Direction::North, Length::Fill, Length::Fixed(THICKNESS));
    let ne = corner(window_id, Direction::NorthEast);

    let w = edge(window_id, Direction::West, Length::Fixed(THICKNESS), Length::Fill);
    let e = edge(window_id, Direction::East, Length::Fixed(THICKNESS), Length::Fill);

    let sw = corner(window_id, Direction::SouthWest);
    let s = edge(window_id, Direction::South, Length::Fill, Length::Fixed(THICKNESS));
    let se = corner(window_id, Direction::SouthEast);

    // Middle row: side handles plus a transparent gap in the centre that lets
    // events fall through to the layer below. `Space` has no interaction and
    // captures no events.
    let middle = row![
        w,
        Space::new().width(Length::Fill).height(Length::Fill),
        e,
    ]
    .height(Length::Fill)
    .width(Length::Fill);

    let top = row![nw, n, ne].width(Length::Fill);
    let bottom = row![sw, s, se].width(Length::Fill);

    column![top, middle, bottom]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn corner(window_id: window::Id, direction: Direction) -> Element<'static, Message> {
    let cursor = corner_cursor(direction);
    mouse_area(
        Space::new()
            .width(Length::Fixed(THICKNESS))
            .height(Length::Fixed(THICKNESS)),
    )
    .interaction(cursor)
    .on_press(Message::WindowDragResize(window_id, direction))
    .into()
}

fn edge(
    window_id: window::Id,
    direction: Direction,
    width: Length,
    height: Length,
) -> Element<'static, Message> {
    let cursor = edge_cursor(direction);
    mouse_area(Space::new().width(width).height(height))
        .interaction(cursor)
        .on_press(Message::WindowDragResize(window_id, direction))
        .into()
}

fn edge_cursor(direction: Direction) -> mouse::Interaction {
    match direction {
        Direction::North | Direction::South => mouse::Interaction::ResizingVertically,
        Direction::East | Direction::West => mouse::Interaction::ResizingHorizontally,
        _ => mouse::Interaction::None,
    }
}

fn corner_cursor(direction: Direction) -> mouse::Interaction {
    match direction {
        // NW ↘ SE diagonal
        Direction::NorthWest | Direction::SouthEast => mouse::Interaction::ResizingDiagonallyDown,
        // NE ↙ SW diagonal
        Direction::NorthEast | Direction::SouthWest => mouse::Interaction::ResizingDiagonallyUp,
        _ => mouse::Interaction::None,
    }
}
