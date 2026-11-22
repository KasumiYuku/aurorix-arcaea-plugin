use std::fmt;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Region {
    Intl,
    Cn,
}

pub const REGIONS: [Region; 2] = [Region::Intl, Region::Cn];

impl Region {
    pub fn as_str(self) -> &'static str {
        match self {
            Region::Intl => "intl",
            Region::Cn => "cn",
        }
    }

    pub fn is_cn(self) -> bool {
        matches!(self, Region::Cn)
    }

    pub fn normalize(text: &str) -> Self {
        Region::from_str(text).unwrap_or(Region::Intl)
    }

    pub fn account_key(self, user_id: &str) -> String {
        format!("acct:{}:{user_id}", self.as_str())
    }

    pub fn legacy_account_key(user_id: &str) -> String {
        format!("acct:{user_id}")
    }

    pub fn snapshot_key(self, user_id: &str) -> String {
        format!("last:b50:{}:{user_id}", self.as_str())
    }
}

impl fmt::Display for Region {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Region {
    type Err = ModelError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text.trim().to_ascii_lowercase().as_str() {
            "intl" | "international" | "global" | "国际服" | "国际" => Ok(Region::Intl),
            "cn" | "china" | "国服" => Ok(Region::Cn),
            other => Err(ModelError::Region(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Difficulty {
    Past,
    Present,
    Future,
    Beyond,
    Eternal,
    Ins,
}

pub fn diff_order_weight(difficulty: Difficulty) -> u8 {
    match difficulty {
        Difficulty::Past => 0,
        Difficulty::Present => 1,
        Difficulty::Future => 2,
        Difficulty::Eternal => 3,
        Difficulty::Beyond => 4,
        Difficulty::Ins => 5,
    }
}

pub fn diff_display_weight(difficulty: Difficulty) -> u8 {
    match difficulty {
        Difficulty::Ins => 0,
        Difficulty::Beyond => 1,
        Difficulty::Eternal => 2,
        Difficulty::Future => 3,
        Difficulty::Present => 4,
        Difficulty::Past => 5,
    }
}

impl Difficulty {
    pub const ALL: [Difficulty; 6] = [
        Difficulty::Past,
        Difficulty::Present,
        Difficulty::Future,
        Difficulty::Beyond,
        Difficulty::Eternal,
        Difficulty::Ins,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Difficulty::Past => "PST",
            Difficulty::Present => "PRS",
            Difficulty::Future => "FTR",
            Difficulty::Beyond => "BYD",
            Difficulty::Eternal => "ETR",
            Difficulty::Ins => "INS",
        }
    }

    pub fn as_upstream(self) -> i16 {
        match self {
            Difficulty::Past => 0,
            Difficulty::Present => 1,
            Difficulty::Future => 2,
            Difficulty::Beyond => 3,
            Difficulty::Ins => 3,
            Difficulty::Eternal => 4,
        }
    }

    pub fn from_upstream(code: i64) -> Option<Self> {
        Difficulty::ALL
            .into_iter()
            .find(|difficulty| i64::from(difficulty.as_upstream()) == code)
    }

    pub fn parse(text: &str) -> Option<Self> {
        let lowered = text.trim().to_lowercase();
        let named = match lowered.as_str() {
            "past" | "过去" => Some(Difficulty::Past),
            "present" | "现在" => Some(Difficulty::Present),
            "future" | "未来" => Some(Difficulty::Future),
            "beyond" | "超越" => Some(Difficulty::Beyond),
            "eternal" | "永恒" => Some(Difficulty::Eternal),
            "inscribed" => Some(Difficulty::Ins),
            other => other
                .parse::<u8>()
                .ok()
                .filter(|slot| *slot <= 4)
                .and_then(|slot| Difficulty::ALL.into_iter().find(|d| d.rank() == slot)),
        };
        named.or_else(|| {
            Difficulty::ALL
                .into_iter()
                .find(|difficulty| lowered == difficulty.as_str().to_lowercase())
        })
    }

    pub fn rank(self) -> u8 {
        self.as_upstream() as u8
    }
}

impl fmt::Display for Difficulty {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Difficulty {
    type Err = ModelError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Difficulty::parse(text).ok_or_else(|| ModelError::Difficulty(text.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Grade {
    Pm,
    Fr,
    ExPlus,
    Ex,
    Aa,
    A,
    B,
    C,
    D,
}

pub const GRADE_THRESHOLDS: [(Grade, i64); 7] = [
    (Grade::D, 0),
    (Grade::C, 8_600_000),
    (Grade::B, 8_900_000),
    (Grade::A, 9_200_000),
    (Grade::Aa, 9_500_000),
    (Grade::Ex, 9_800_000),
    (Grade::ExPlus, 9_900_000),
];

impl Grade {
    pub const ALL: [Grade; 9] = [
        Grade::Pm,
        Grade::Fr,
        Grade::ExPlus,
        Grade::Ex,
        Grade::Aa,
        Grade::A,
        Grade::B,
        Grade::C,
        Grade::D,
    ];

    pub const SCORE: [Grade; 7] = [
        Grade::ExPlus,
        Grade::Ex,
        Grade::Aa,
        Grade::A,
        Grade::B,
        Grade::C,
        Grade::D,
    ];

    pub fn from_score(score: i64) -> Self {
        let mut grade = Grade::D;
        for (candidate, min) in GRADE_THRESHOLDS {
            if score >= min {
                grade = candidate;
            }
        }
        grade
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Grade::Pm => "PM",
            Grade::Fr => "FR",
            Grade::ExPlus => "EX+",
            Grade::Ex => "EX",
            Grade::Aa => "AA",
            Grade::A => "A",
            Grade::B => "B",
            Grade::C => "C",
            Grade::D => "D",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_uppercase().as_str() {
            "PM" | "PUREMEMORY" | "PURE_MEMORY" => Some(Grade::Pm),
            "FR" | "FULLRECALL" | "FULL_RECALL" => Some(Grade::Fr),
            "EX+" | "EXPLUS" | "EX_PLUS" => Some(Grade::ExPlus),
            "EX" => Some(Grade::Ex),
            "AA" => Some(Grade::Aa),
            "A" => Some(Grade::A),
            "B" => Some(Grade::B),
            "C" => Some(Grade::C),
            "D" => Some(Grade::D),
            _ => None,
        }
    }
}

impl fmt::Display for Grade {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Grade {
    type Err = ModelError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Grade::parse(text).ok_or_else(|| ModelError::Grade(text.to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub region: Region,
    pub user_id: String,
    pub arcaea_uid: Option<i64>,
    pub friend_code: String,
    pub name: String,
    pub bound_at: DateTime<Utc>,
}

impl Account {
    pub fn new(region: Region, user_id: impl Into<String>, arcaea_uid: i64) -> Self {
        Self {
            region,
            user_id: user_id.into(),
            arcaea_uid: (arcaea_uid > 0).then_some(arcaea_uid),
            friend_code: String::new(),
            name: String::new(),
            bound_at: Utc::now(),
        }
    }

    pub fn is_pending(&self) -> bool {
        self.arcaea_uid.is_none()
    }

    pub fn uid(&self) -> Option<i64> {
        self.arcaea_uid.filter(|uid| *uid > 0)
    }

    pub fn needs_friend_code_resolve(&self) -> bool {
        self.arcaea_uid.is_none() && !self.friend_code.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skin {
    pub char_id: i32,
    pub uncapped: bool,
    pub override_form: bool,
    pub banner: String,
    pub seen_at: Option<DateTime<Utc>>,
}

impl Skin {
    pub const UNKNOWN_CHAR: i32 = -1;

    pub fn unknown() -> Self {
        Self {
            char_id: Skin::UNKNOWN_CHAR,
            uncapped: false,
            override_form: false,
            banner: String::new(),
            seen_at: None,
        }
    }

    pub fn known(&self) -> bool {
        self.char_id >= 0 && self.seen_at.is_some()
    }
}

impl Default for Skin {
    fn default() -> Self {
        Skin::unknown()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Prefs {
    pub char: String,
    pub banner: String,
    pub theme: String,
}

impl Prefs {
    pub fn is_empty(&self) -> bool {
        self.char.is_empty() && self.banner.is_empty() && self.theme.is_empty()
    }

    pub fn observable(&self) -> bool {
        !self.char.is_empty() || !self.banner.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    #[error("未知的区域: {0}")]
    Region(String),
    #[error("未知的难度: {0}")]
    Difficulty(String),
    #[error("未知的评级: {0}")]
    Grade(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_001() {
        assert_eq!(Region::normalize("CN"), Region::Cn);
        assert_eq!(Region::normalize("国服"), Region::Cn);
        assert_eq!(Region::normalize("intl"), Region::Intl);
        assert_eq!(Region::normalize("火星服"), Region::Intl);
        assert_eq!("国际服".parse::<Region>(), Ok(Region::Intl));
    }

    #[test]
    fn case_002() {
        assert_eq!(Region::Intl.account_key("u1"), "acct:intl:u1");
        assert_eq!(Region::Cn.account_key("u1"), "acct:cn:u1");
        assert_eq!(Region::legacy_account_key("u1"), "acct:u1");
        assert_eq!(Region::Intl.snapshot_key("u1"), "last:b50:intl:u1");
    }

    #[test]
    fn case_003() {
        assert_eq!(Difficulty::parse("ftr"), Some(Difficulty::Future));
        assert_eq!(Difficulty::parse("Present"), Some(Difficulty::Present));
        assert_eq!(Difficulty::parse("3"), Some(Difficulty::Beyond));
        assert_eq!(Difficulty::parse("ETR"), Some(Difficulty::Eternal));
        assert_eq!(Difficulty::parse("PST"), Some(Difficulty::Past));
        assert_eq!(Difficulty::parse("无限"), None);
    }

    #[test]
    fn case_004() {
        for difficulty in Difficulty::ALL {
            let code = i64::from(difficulty.as_upstream());
            if difficulty == Difficulty::Ins {
                assert_eq!(code, 3, "INS 没有自己的协议序号，发请求前折回 BYD 槽位");
                continue;
            }
            assert_eq!(Difficulty::from_upstream(code), Some(difficulty));
            assert_eq!(difficulty.rank(), difficulty.as_upstream() as u8);
        }
        assert_eq!(Difficulty::from_upstream(3), Some(Difficulty::Beyond));
        assert_eq!(Difficulty::from_upstream(9), None);
    }

    #[test]
    fn case_005() {
        assert_eq!(Grade::parse("ex+"), Some(Grade::ExPlus));
        assert_eq!("EX".parse::<Grade>(), Ok(Grade::Ex));
        assert_eq!(Grade::parse("d"), Some(Grade::D));
        assert_eq!(Grade::parse("aa"), Some(Grade::Aa));
        assert!("AA".parse::<Grade>().is_ok());
        assert!(Grade::Ex < Grade::Aa);
        assert!(Grade::Aa < Grade::A);
        assert_eq!(Grade::ALL.len(), 9);
        assert_eq!(Grade::SCORE.len(), 7);
        assert_eq!(Grade::parse("pm"), Some(Grade::Pm));
        assert_eq!(Grade::parse("FR"), Some(Grade::Fr));
        assert!(Grade::Pm < Grade::Fr);
        assert!(Grade::Fr < Grade::ExPlus);
        assert_eq!(Grade::Aa.to_string(), "AA");
        assert!(Grade::ExPlus < Grade::Ex);
        assert!(Grade::Ex < Grade::D);
        assert_eq!(Grade::A.to_string(), "A");
    }

    #[test]
    fn grade_from_score_uses_seven_bands() {
        for (score, want) in [
            (0, Grade::D),
            (8_599_999, Grade::D),
            (8_600_000, Grade::C),
            (8_900_000, Grade::B),
            (9_200_000, Grade::A),
            (9_500_000, Grade::Aa),
            (9_800_000, Grade::Ex),
            (9_899_999, Grade::Ex),
            (9_900_000, Grade::ExPlus),
            (10_000_000, Grade::ExPlus),
        ] {
            assert_eq!(Grade::from_score(score), want, "分数 {score} 的分数评级");
        }
        assert_eq!(GRADE_THRESHOLDS.len(), 7, "分数评级只有七档");
        let mut previous = -1;
        for (_, min) in GRADE_THRESHOLDS {
            assert!(min > previous, "阈值必须由低到高");
            previous = min;
        }
        for (grade, _) in GRADE_THRESHOLDS {
            assert!(
                Grade::SCORE.contains(&grade),
                "{grade} 不是分数评级, 不许进阈值表"
            );
        }
    }

    #[test]
    fn case_006() {
        let mut account = Account::new(Region::Intl, "u1", 0);
        assert!(account.is_pending(), "uid 为 0 表示还没解析出账号");
        account.arcaea_uid = None;
        assert!(!account.needs_friend_code_resolve(), "没有好友码就无从解析");
        account.friend_code = "431343680".to_string();
        assert!(account.needs_friend_code_resolve());
        assert_eq!(account.region.account_key(&account.user_id), "acct:intl:u1");
    }

    #[test]
    fn case_007() {
        let mut skin = Skin::unknown();
        assert!(!skin.known());
        skin.char_id = 12;
        assert!(!skin.known(), "缺观测时间仍然算没见过");
        skin.seen_at = Some(Utc::now());
        assert!(skin.known());
        assert_eq!(Skin::default(), Skin::unknown());
    }

    #[test]
    fn case_008() {
        let prefs = Prefs::default();
        assert!(prefs.is_empty());
        let mut prefs = prefs;
        prefs.theme = "arknights".to_string();
        assert!(!prefs.is_empty());
        assert!(!prefs.observable(), "只有主题不算渲染素材偏好");
        prefs.char = "char_12.png".to_string();
        assert!(prefs.observable());
    }

    #[test]
    fn case_009() {
        let mut order = Difficulty::ALL.to_vec();
        order.sort_by_key(|difficulty| diff_order_weight(*difficulty));
        let names: Vec<&str> = order.iter().map(|difficulty| difficulty.as_str()).collect();
        assert_eq!(names, vec!["PST", "PRS", "FTR", "ETR", "BYD", "INS"]);
        let mut display = Difficulty::ALL.to_vec();
        display.sort_by_key(|difficulty| diff_display_weight(*difficulty));
        let names: Vec<&str> = display.iter().map(|difficulty| difficulty.as_str()).collect();
        assert_eq!(names, vec!["INS", "BYD", "ETR", "FTR", "PRS", "PST"]);
    }
}
