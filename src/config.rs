// キー配列のマッピング
//   jp -> us
// キーを複数の別イベントにマッピング
//   ctrl-[ -> ctrl-[, nonconvert
// 専用メタキーの作成
//   ralt -> ohk_meta
// 専用メタキーを使ったマッピング
//   ohk_meta + a -> home
//   ohk_meta + g -> end
//   ohk_meta + s -> left
//   ohk_meta + d -> down
//   ohk_meta + f -> right
//   ohk_meta + e -> up
// 空打ちでのマッピング
//   lalt -> nonconvert
//   ohk_meta -> convert

// 右 alt を潰して使用されていないキーコードを割り当てる
// キーコードの一覧はここ https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub const OHK_META: u16 = 0x0f;

const UNDEFINED: u16 = 0x07;
const LEFT_ALT: u16 = 0xa4;
const RIGHT_ALT: u16 = 0xa5;
const HENKAN: u16 = 0x1c;
const MUHENKAN: u16 = 0x1d;

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
pub fn just_down_up(code: u16) -> Option<Vec<Event>> {
    match code {
        // OHK_META: OHK_META のあとに変換
        OHK_META => Some(vec![
            Event::new(OHK_META, State::UP),
            Event::new(HENKAN, State::DOWN),
            Event::new(HENKAN, State::UP),
        ]),
        // left alt: left alt のあとに無変換
        //           undefined を挟むことでメニューにカーソルが吸われるのを抑制する
        LEFT_ALT => Some(vec![
            Event::new(UNDEFINED, State::DOWN),
            Event::new(UNDEFINED, State::UP),
            Event::new(LEFT_ALT, State::UP),
            Event::new(MUHENKAN, State::DOWN),
            Event::new(MUHENKAN, State::UP),
        ]),
        _ => None,
    }
}
