pub mod avatar;
pub mod brand;
pub mod b50;
pub mod diff;
pub mod gallery;
pub mod guess;
pub(crate) mod lagged_rng;
pub mod harness;
pub mod jacket_pool;
pub mod lists;
pub(crate) mod memo;
pub mod player;
pub mod score;
pub mod search;
pub mod select;
pub mod sketch;
pub mod song;
pub mod stats;
pub mod theme;
pub mod widgets;

use crate::domain::BoardKind;

pub const CARD_KIND_COUNT: usize = 19;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CardKind {
    B50,
    Score,
    Recent,
    Song,
    Rand,
    Search,
    Guess,
    List,
    Table,
    Ahv,
    Stats,
    Suggest,
    Chap,
    Trend,
    His,
    Me,
    Char,
    Banner,
    Theme,
}

impl CardKind {
    pub const ALL: [CardKind; CARD_KIND_COUNT] = [
        CardKind::B50,
        CardKind::Score,
        CardKind::Recent,
        CardKind::Song,
        CardKind::Rand,
        CardKind::Search,
        CardKind::Guess,
        CardKind::List,
        CardKind::Table,
        CardKind::Ahv,
        CardKind::Stats,
        CardKind::Suggest,
        CardKind::Chap,
        CardKind::Trend,
        CardKind::His,
        CardKind::Me,
        CardKind::Char,
        CardKind::Banner,
        CardKind::Theme,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            CardKind::B50 => "b50",
            CardKind::Score => "score",
            CardKind::Recent => "recent",
            CardKind::Song => "song",
            CardKind::Rand => "rand",
            CardKind::Search => "search",
            CardKind::Guess => "guess",
            CardKind::List => "list",
            CardKind::Table => "table",
            CardKind::Ahv => "ahv",
            CardKind::Stats => "stats",
            CardKind::Suggest => "suggest",
            CardKind::Chap => "chap",
            CardKind::Trend => "trend",
            CardKind::His => "his",
            CardKind::Me => "me",
            CardKind::Char => "char",
            CardKind::Banner => "banner",
            CardKind::Theme => "theme",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            CardKind::B50 => "B30 与最近成绩榜面",
            CardKind::Score => "单曲成绩",
            CardKind::Recent => "最近结算",
            CardKind::Song => "曲目档案",
            CardKind::Rand => "随机选曲",
            CardKind::Search => "曲目检索",
            CardKind::Guess => "猜曲绘",
            CardKind::List => "曲目清单",
            CardKind::Table => "定数表",
            CardKind::Ahv => "成就表",
            CardKind::Stats => "区间成绩统计",
            CardKind::Suggest => "推分建议",
            CardKind::Chap => "曲包一览",
            CardKind::Trend => "成绩趋势",
            CardKind::His => "单曲历史",
            CardKind::Me => "玩家档案",
            CardKind::Char => "立绘墙",
            CardKind::Banner => "横幅墙",
            CardKind::Theme => "主题一览",
        }
    }

    pub fn parse(text: &str) -> Option<CardKind> {
        let wanted = text.trim().to_ascii_lowercase();
        CardKind::ALL
            .into_iter()
            .find(|kind| kind.as_str() == wanted)
    }

    pub fn spec(self) -> CardSpec {
        match self {
            CardKind::B50 => CardSpec::new(self, Extent::fixed(1820), Extent::fixed(2134)),
            CardKind::Score => CardSpec::new(self, Extent::fixed(2016), Extent::fixed(1333)),
            CardKind::Recent => CardSpec::new(self, Extent::fixed(2016), Extent::fixed(1333)),
            CardKind::Song => {
                CardSpec::new(self, Extent::set(&[1100, 1448, 1796]), Extent::fixed(770))
            }
            CardKind::Rand => CardSpec::new(self, Extent::fixed(2376), Extent::fixed(1080)),
            CardKind::Search => {
                CardSpec::new(self, Extent::dynamic(360, 2400), Extent::dynamic(360, 4400))
            }
            CardKind::Guess => {
                CardSpec::new(self, Extent::set(&[924, 739]), Extent::set(&[1257, 1006]))
            }
            CardKind::List => CardSpec::new(self, Extent::fixed(1188), Extent::dynamic(360, 12000)),
            CardKind::Table => {
                CardSpec::new(self, Extent::fixed(1409), Extent::dynamic(360, 12000))
            }
            CardKind::Ahv => CardSpec::new(self, Extent::fixed(1600), Extent::dynamic(360, 12000)),
            CardKind::Stats => {
                CardSpec::new(self, Extent::fixed(1080), Extent::dynamic(360, 12000))
            }
            CardKind::Suggest => {
                CardSpec::new(self, Extent::fixed(1400), Extent::dynamic(360, 12000))
            }
            CardKind::Chap => CardSpec::new(self, Extent::fixed(1350), Extent::dynamic(360, 12000)),
            CardKind::Trend => {
                CardSpec::new(self, Extent::fixed(1080), Extent::dynamic(360, 12000))
            }
            CardKind::His => CardSpec::new(self, Extent::fixed(1080), Extent::dynamic(360, 12000)),
            CardKind::Me => CardSpec::new(self, Extent::fixed(2800), Extent::fixed(1900)),
            CardKind::Char => CardSpec::new(self, Extent::fixed(2016), Extent::dynamic(360, 12000)),
            CardKind::Banner => {
                CardSpec::new(self, Extent::fixed(2016), Extent::dynamic(360, 12000))
            }
            CardKind::Theme => {
                CardSpec::new(self, Extent::fixed(2016), Extent::dynamic(360, 12000))
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extent {
    Fixed(i32),
    Set(&'static [i32]),
    Dynamic { min: i32, max: i32 },
}

impl Extent {
    pub fn fixed(value: i32) -> Self {
        Extent::Fixed(value)
    }

    pub fn set(values: &'static [i32]) -> Self {
        Extent::Set(values)
    }

    pub fn dynamic(min: i32, max: i32) -> Self {
        let min = min.max(1);
        Extent::Dynamic {
            min,
            max: max.max(min),
        }
    }

    pub fn accepts(self, value: i32) -> bool {
        match self {
            Extent::Fixed(fixed) => value == fixed,
            Extent::Set(values) => values.contains(&value),
            Extent::Dynamic { min, max } => (min..=max).contains(&value),
        }
    }

    pub fn is_fixed(self) -> bool {
        matches!(self, Extent::Fixed(_) | Extent::Set(_))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CardSpec {
    pub kind: CardKind,
    pub width: Extent,
    pub height: Extent,
}

impl CardSpec {
    pub const fn new(kind: CardKind, width: Extent, height: Extent) -> Self {
        Self {
            kind,
            width,
            height,
        }
    }

    pub fn accepts(self, width: i32, height: i32) -> bool {
        self.width.accepts(width) && self.height.accepts(height)
    }

    pub fn label(self) -> &'static str {
        self.kind.label()
    }
}

pub fn spec_of(kind: CardKind) -> CardSpec {
    kind.spec()
}

pub const BOARD_KINDS: [BoardKind; 6] = BoardKind::ALL;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn case_275() {
        assert_eq!(CardKind::ALL.len(), CARD_KIND_COUNT);
        let mut names = HashSet::new();
        let mut labels = HashSet::new();
        for kind in CardKind::ALL {
            assert!(names.insert(kind.as_str()), "卡片名重复: {}", kind.as_str());
            assert!(labels.insert(kind.label()), "卡片中文名重复");
            assert_eq!(CardKind::parse(kind.as_str()), Some(kind));
        }
        assert_eq!(names.len(), 19);
    }

    #[test]
    fn case_276() {
        let b50 = CardKind::B50.spec();
        assert_eq!(b50.height, Extent::fixed(2134));
        assert_eq!(b50.width, Extent::fixed(1820));
        assert_eq!(CardKind::Me.spec().width, Extent::fixed(2800));
        assert_eq!(CardKind::Me.spec().height, Extent::fixed(1900));
        assert_eq!(
            CardKind::Rand.spec(),
            CardSpec::new(CardKind::Rand, Extent::fixed(2376), Extent::fixed(1080))
        );
        assert_eq!(CardKind::Song.spec().height, Extent::fixed(770));
        assert_eq!(CardKind::Score.spec().width, Extent::fixed(2016));
        assert_eq!(CardKind::Score.spec().height, Extent::fixed(1333));
        assert_eq!(CardKind::Table.spec().width, Extent::fixed(1409));
        assert!(CardKind::Song.spec().width.accepts(1100));
        assert!(CardKind::Song.spec().width.accepts(1448));
        assert!(CardKind::Song.spec().width.accepts(1796));
        assert!(!CardKind::Song.spec().width.accepts(1200));
        assert!(CardKind::Guess.spec().accepts(924, 1257));
        assert!(CardKind::Guess.spec().accepts(739, 1006));
        assert!(!CardKind::Guess.spec().height.accepts(1333));
    }

    #[test]
    fn case_277() {
        for kind in [
            CardKind::List,
            CardKind::Table,
            CardKind::Ahv,
            CardKind::Stats,
            CardKind::Suggest,
            CardKind::Chap,
            CardKind::Trend,
            CardKind::His,
            CardKind::Char,
            CardKind::Banner,
            CardKind::Theme,
        ] {
            let spec = kind.spec();
            assert!(
                !spec.height.is_fixed(),
                "{} 的高度必须按内容算",
                spec.kind.as_str()
            );
            assert!(
                spec.height.accepts(1200),
                "{} 要接受内容撑高",
                spec.kind.as_str()
            );
            assert!(
                !spec.height.accepts(10),
                "{} 不能接受比下限还矮的画布",
                spec.kind.as_str()
            );
        }
    }

    #[test]
    fn case_278() {
        assert_eq!(CardKind::parse("render_song_card"), None);
        assert_eq!(CardKind::parse(""), None);
        assert_eq!(CardKind::parse(" B50 "), Some(CardKind::B50));
    }
}
