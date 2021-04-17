use super::{key_event::KeyEvent, keys, mapping};

#[derive(Debug)]
pub struct State {
    pub last_event: Option<KeyEvent>,
    pub sending_same_key: bool,
    pub ohk_meta_pressed: bool,
    pub left_ctrl_pressed: bool,
    pub right_ctrl_pressed: bool,
}

impl State {
    pub fn update(&mut self, event: KeyEvent) {
        // 装飾キーの状態を保持しておく
        // キーが複数ある場合は正しく状態を保てない場合があるけどとりあえずこれで実用できる
        match event.vk_code() {
            keys::LEFT_CTRL => self.left_ctrl_pressed = !event.is_up(),
            keys::RIGHT_CTRL => self.right_ctrl_pressed = !event.is_up(),
            keys::OHK_META => self.ohk_meta_pressed = !event.is_up(),
            _ => {}
        }

        self.last_event = Some(event);
    }

    pub fn as_mapping_state(&self, event: &KeyEvent) -> mapping::State {
        mapping::State {
            left_ctrl: self.left_ctrl_pressed,
            right_ctrl: self.right_ctrl_pressed,
            ohk_meta: self.ohk_meta_pressed,
            just_down_up: self.last_event.is_some()
                && self.last_event.as_ref().unwrap().vk_code() == event.vk_code(),
        }
    }
}
