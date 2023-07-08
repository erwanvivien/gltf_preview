use crate::{KeyState, UserAction};

impl crate::InputManager {
    fn is_action_pressed(&self, action: UserAction) -> bool {
        self.key_state.get(&action).map(KeyState::to_value) == Some(1f32)
            || self.mouse_state.get(&action).map(KeyState::to_value) == Some(1f32)
    }

    fn get_action_value(&self, action: UserAction) -> f32 {
        self.key_state.get(&action).map_or(0f32, KeyState::to_value)
    }

    #[must_use]
    pub fn get_direction(&self) -> (f32, f32, f32) {
        let forward = self.get_action_value(UserAction::Forward);
        let backward = self.get_action_value(UserAction::Backward);

        let up = self.get_action_value(UserAction::Up);
        let down = self.get_action_value(UserAction::Down);

        let left = self.get_action_value(UserAction::Left);
        let right = self.get_action_value(UserAction::Right);

        debug_assert!((0f32..=1f32).contains(&forward));
        debug_assert!((0f32..=1f32).contains(&backward));
        debug_assert!((0f32..=1f32).contains(&up));
        debug_assert!((0f32..=1f32).contains(&down));
        debug_assert!((0f32..=1f32).contains(&left));
        debug_assert!((0f32..=1f32).contains(&right));

        (forward - backward, up - down, left - right)
    }

    fn get_mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    #[must_use]
    pub fn consume_mouse_delta(&mut self) -> (f64, f64) {
        let delta = self.get_mouse_delta();
        self.mouse_delta = (0f64, 0f64);

        delta
    }

    #[must_use]
    pub fn escape_pressed(&self) -> bool {
        self.is_action_pressed(UserAction::Escape)
    }

    #[must_use]
    pub fn left_click_pressed(&self) -> bool {
        self.is_action_pressed(UserAction::LeftClick)
    }

    #[must_use]
    pub fn right_click_pressed(&self) -> bool {
        self.is_action_pressed(UserAction::RightClick)
    }
}
