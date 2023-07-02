use crate::{KeyState, UserAction};

impl crate::InputManager {
    fn is_action_pressed(&self, action: UserAction) -> bool {
        self.key_state.get(&action).map(KeyState::to_value) == Some(1f32)
            || self.mouse_state.get(&action).map(KeyState::to_value) == Some(1f32)
    }

    fn get_action_value(&self, action: UserAction) -> f32 {
        self.key_state
            .get(&action)
            .map(KeyState::to_value)
            .unwrap_or(0f32)
    }

    pub fn get_direction(&self) -> (f32, f32, f32) {
        let forward = self.get_action_value(UserAction::Forward);
        let backward = self.get_action_value(UserAction::Backward);

        let up = self.get_action_value(UserAction::Up);
        let down = self.get_action_value(UserAction::Down);

        let left = self.get_action_value(UserAction::Left);
        let right = self.get_action_value(UserAction::Right);

        debug_assert!(forward <= 1f32 && forward >= 0f32);
        debug_assert!(backward <= 1f32 && backward >= 0f32);
        debug_assert!(up <= 1f32 && up >= 0f32);
        debug_assert!(down <= 1f32 && down >= 0f32);
        debug_assert!(left <= 1f32 && left >= 0f32);
        debug_assert!(right <= 1f32 && right >= 0f32);

        (forward - backward, up - down, left - right)
    }

    fn get_mouse_delta(&self) -> (f64, f64) {
        self.mouse_delta
    }

    pub fn consume_mouse_delta(&mut self) -> (f64, f64) {
        let delta = self.get_mouse_delta();
        self.mouse_delta = (0f64, 0f64);

        delta
    }

    pub fn escape_pressed(&self) -> bool {
        self.is_action_pressed(UserAction::Escape)
    }

    pub fn left_click_pressed(&self) -> bool {
        self.is_action_pressed(UserAction::LeftClick)
    }

    pub fn right_click_pressed(&self) -> bool {
        self.is_action_pressed(UserAction::RightClick)
    }
}
