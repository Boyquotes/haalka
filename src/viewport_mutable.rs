use crate::Scrollable;
use bevy::prelude::*;
use futures_signals::signal::Signal;

pub trait ViewportMutable: Scrollable {
    // TODO
    // fn on_viewport_location_change(self, mut handler: impl FnMut(Scene, Viewport) + 'static) -> Self

    fn viewport_x_signal<S: Signal<Item = f32> + Send + 'static>(
        mut self,
        x_signal_option: impl Into<Option<S>>,
    ) -> Self {
        if let Some(x_signal) = x_signal_option.into() {
            self = self.update_raw_el(|raw_el| {
                raw_el.on_signal_one_shot(
                    x_signal,
                    |In((entity, y)): In<(Entity, f32)>,
                     // TODO: combining these queries might be better?
                     mut style_query: Query<&mut Style>,
                     parent_query: Query<&Parent>,
                     node_query: Query<&Node>| {
                        let Ok(width) = node_query.get(entity).map(|node| node.size().x) else {
                            return;
                        };
                        let Ok(parent) = parent_query.get(entity) else { return };
                        let container_width = node_query.get(parent.get()).unwrap().size().y;
                        let max_scroll: f32 = (width - container_width).max(0.);
                        let Ok(mut style) = style_query.get_mut(entity) else {
                            return;
                        };
                        style.left = Val::Px(y.clamp(-max_scroll, 0.));
                    },
                )
            });
        }
        self
    }

    fn viewport_y_signal<S: Signal<Item = f32> + Send + 'static>(
        mut self,
        y_signal_option: impl Into<Option<S>>,
    ) -> Self {
        if let Some(y_signal) = y_signal_option.into() {
            self = self.update_raw_el(|raw_el| {
                raw_el.on_signal_one_shot(
                    y_signal,
                    |In((entity, y)): In<(Entity, f32)>,
                     // TODO: combining these queries might be better?
                     mut style_query: Query<&mut Style>,
                     parent_query: Query<&Parent>,
                     node_query: Query<&Node>| {
                        let Ok(height) = node_query.get(entity).map(|node| node.size().y) else {
                            return;
                        };
                        let Ok(parent) = parent_query.get(entity) else { return };
                        let container_height = node_query.get(parent.get()).unwrap().size().y;
                        let max_scroll: f32 = (height - container_height).max(0.);
                        let Ok(mut style) = style_query.get_mut(entity) else {
                            return;
                        };
                        style.top = Val::Px(y.clamp(-max_scroll, 0.));
                    },
                )
            });
        }
        self
    }
}
