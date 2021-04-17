use super::keys;

#[derive(Debug, PartialEq, Eq)]
pub enum UD {
    UP,
    DOWN,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    pub code: u16,
    pub state: UD,
}

impl Event {
    pub fn new(code: u16, state: UD) -> Self {
        Self { code, state }
    }
}

#[derive(Debug)]
pub struct State {
    pub left_ctrl: bool,
    pub right_ctrl: bool,
    pub ohk_meta: bool,
    pub just_down_up: bool,
}

impl State {
    pub fn ctrl(&self) -> bool {
        self.left_ctrl || self.right_ctrl
    }
}

pub fn map_simply(code: u16) -> Option<u16> {
    match code {
        keys::LEFT_ALT => Some(keys::LEFT_ALT), // 自前のイベントで上書きしておかないと up を書き換えたときに押しっぱなし判定になってしまう
        keys::RIGHT_ALT => Some(keys::OHK_META), // 独自のメタキーに割り当てる
        _ => None,
    }
}

pub fn map(code: u16, state: &State) -> Option<u16> {
    // 現時点では OHK_META と一緒に使うやつしかないのでスッキリさせるために先に return しておく
    if !state.ohk_meta {
        return None;
    }

    match code {
        keys::A => Some(keys::HOME),
        keys::S => Some(keys::LEFT),
        keys::D => Some(keys::DOWN),
        keys::F => Some(keys::RIGHT),
        keys::E => Some(keys::UP),
        keys::G => Some(keys::END),
        _ => None,
    }
}

pub fn map_on_up(code: u16, state: &State) -> Option<Vec<Event>> {
    match code {
        // OHK_META: OHK_META のあとに変換
        keys::OHK_META => {
            if state.just_down_up {
                Some(vec![
                    Event::new(keys::OHK_META, UD::UP),
                    Event::new(keys::HENKAN, UD::DOWN),
                    Event::new(keys::HENKAN, UD::UP),
                ])
            } else {
                None
            }
        }
        // left alt: left alt のあとに無変換
        //           undefined を挟むことでメニューにカーソルが吸われるのを抑制する
        keys::LEFT_ALT => {
            if state.just_down_up {
                Some(vec![
                    Event::new(keys::UNDEFINED, UD::DOWN),
                    Event::new(keys::UNDEFINED, UD::UP),
                    Event::new(keys::LEFT_ALT, UD::UP),
                    Event::new(keys::MUHENKAN, UD::DOWN),
                    Event::new(keys::MUHENKAN, UD::UP),
                ])
            } else {
                None
            }
        }
        keys::OPEN_BRACKET => {
            if state.ctrl() {
                // ctrl-[: ctrl-[ のあとに無変換
                Some(vec![
                    Event::new(keys::OPEN_BRACKET, UD::UP),
                    Event::new(keys::MUHENKAN, UD::DOWN),
                    Event::new(keys::MUHENKAN, UD::UP),
                ])
            } else {
                None
            }
        }
        _ => None,
    }
}
