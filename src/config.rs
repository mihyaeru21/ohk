// 残り
// キー配列のマッピング
//   jp -> us
// キーを複数の別イベントにマッピング
//   ctrl-[ -> ctrl-[, nonconvert

// 右 alt を潰して使用されていないキーコードを割り当てる
// キーコードの一覧はここ https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub const KEY_OHK_META: u16 = 0x0f;

pub const KEY_LEFT_CTRL: u16 = 0xa2;
pub const KEY_RIGHT_CTRL: u16 = 0xa3;

const KEY_UNDEFINED: u16 = 0x07;
const KEY_LEFT_ALT: u16 = 0xa4;
const KEY_RIGHT_ALT: u16 = 0xa5;
const KEY_HENKAN: u16 = 0x1c;
const KEY_MUHENKAN: u16 = 0x1d;
const KEY_END: u16 = 0x23;
const KEY_HOME: u16 = 0x24;
const KEY_LEFT: u16 = 0x25;
const KEY_UP: u16 = 0x26;
const KEY_RIGHT: u16 = 0x27;
const KEY_DOWN: u16 = 0x28;
const KEY_A: u16 = 0x41;
const KEY_D: u16 = 0x44;
const KEY_E: u16 = 0x45;
const KEY_F: u16 = 0x46;
const KEY_G: u16 = 0x47;
const KEY_S: u16 = 0x53;
const KEY_OPEN_BRACKET: u16 = 0xdb;

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

// TODO: rename
pub fn simple_map(code: u16) -> Option<u16> {
    match code {
        KEY_LEFT_ALT => Some(KEY_LEFT_ALT), // 自前のイベントで上書きしておかないと up を書き換えたときに押しっぱなし判定になってしまう
        KEY_RIGHT_ALT => Some(KEY_OHK_META), // 独自のメタキーに割り当てる
        _ => None,
    }
}

// TODO: rename
pub fn simple_map_with_meta(code: u16) -> Option<u16> {
    match code {
        KEY_A => Some(KEY_HOME),
        KEY_S => Some(KEY_LEFT),
        KEY_D => Some(KEY_DOWN),
        KEY_F => Some(KEY_RIGHT),
        KEY_E => Some(KEY_UP),
        KEY_G => Some(KEY_END),
        _ => None,
    }
}

// TODO: rename
pub fn map_on_up(code: u16, state: &State) -> Option<Vec<Event>> {
    match code {
        // OHK_META: OHK_META のあとに変換
        KEY_OHK_META => {
            if state.just_down_up {
                Some(vec![
                    Event::new(KEY_OHK_META, UD::UP),
                    Event::new(KEY_HENKAN, UD::DOWN),
                    Event::new(KEY_HENKAN, UD::UP),
                ])
            } else {
                None
            }
        }
        // left alt: left alt のあとに無変換
        //           undefined を挟むことでメニューにカーソルが吸われるのを抑制する
        KEY_LEFT_ALT => {
            if state.just_down_up {
                Some(vec![
                    Event::new(KEY_UNDEFINED, UD::DOWN),
                    Event::new(KEY_UNDEFINED, UD::UP),
                    Event::new(KEY_LEFT_ALT, UD::UP),
                    Event::new(KEY_MUHENKAN, UD::DOWN),
                    Event::new(KEY_MUHENKAN, UD::UP),
                ])
            } else {
                None
            }
        }
        KEY_OPEN_BRACKET => {
            if state.ctrl() {
                // ctrl-[: ctrl-[ のあとに無変換
                Some(vec![
                    Event::new(KEY_OPEN_BRACKET, UD::UP),
                    Event::new(KEY_MUHENKAN, UD::DOWN),
                    Event::new(KEY_MUHENKAN, UD::UP),
                ])
            } else {
                None
            }
        }
        _ => None,
    }
}
