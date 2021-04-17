// 残り
// キー配列のマッピング
//   jp -> us
// キーを複数の別イベントにマッピング
//   ctrl-[ -> ctrl-[, nonconvert

// 右 alt を潰して使用されていないキーコードを割り当てる
// キーコードの一覧はここ https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub const OHK_META: u16 = 0x0f;

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

#[derive(Debug, PartialEq, Eq)]
pub enum State {
    UP,
    DOWN,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    pub code: u16,
    pub state: State,
}

impl Event {
    pub fn new(code: u16, state: State) -> Self {
        Self { code, state }
    }
}

// TODO: rename
pub fn simple_map(code: u16) -> Option<u16> {
    match code {
        KEY_LEFT_ALT => Some(KEY_LEFT_ALT), // 自前のイベントで上書きしておかないと up を書き換えたときに押しっぱなし判定になってしまう
        KEY_RIGHT_ALT => Some(OHK_META),    // 独自のメタキーに割り当てる
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
pub fn just_down_up(code: u16) -> Option<Vec<Event>> {
    match code {
        // OHK_META: OHK_META のあとに変換
        OHK_META => Some(vec![
            Event::new(OHK_META, State::UP),
            Event::new(KEY_HENKAN, State::DOWN),
            Event::new(KEY_HENKAN, State::UP),
        ]),
        // left alt: left alt のあとに無変換
        //           undefined を挟むことでメニューにカーソルが吸われるのを抑制する
        KEY_LEFT_ALT => Some(vec![
            Event::new(KEY_UNDEFINED, State::DOWN),
            Event::new(KEY_UNDEFINED, State::UP),
            Event::new(KEY_LEFT_ALT, State::UP),
            Event::new(KEY_MUHENKAN, State::DOWN),
            Event::new(KEY_MUHENKAN, State::UP),
        ]),
        _ => None,
    }
}
