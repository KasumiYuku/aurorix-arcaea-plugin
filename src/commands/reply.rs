use std::collections::BTreeMap;
use std::sync::OnceLock;

use aurorix_plugin::{Outgoing, Part, RenderCard, ReplyKind};
use aurorix_template::{Arg, Args};

use crate::commands::{buttons, hints};
use crate::domain::{Song, SongHit, SongMatch};
use crate::model::Region;

static ONCE: OnceLock<()> = OnceLock::new();

fn ready() {
    ONCE.get_or_init(|| {
        crate::templates::register_templates(aurorix_template::Registry::global());
    });
}

pub fn args() -> Args {
    ready();
    Args::new()
}

pub fn put(args: &mut Args, key: &str, value: impl Into<String>) {
    args.insert(key.to_string(), Arg::Str(value.into()));
}

pub fn row(args: &mut Args, key: &str, fields: &[(&str, String)]) {
    let mut item = BTreeMap::new();
    for (name, value) in fields {
        item.insert(name.to_string(), Arg::Str(value.clone()));
    }
    let entry = Arg::Map(item);
    match args.get_mut(key) {
        Some(Arg::List(items)) => items.push(entry),
        Some(other) => *other = Arg::List(vec![entry]),
        None => {
            args.insert(key.to_string(), Arg::List(vec![entry]));
        }
    }
}

pub fn note(args: &mut Args, text: impl Into<String>) {
    row(args, "notes", &[("text", text.into())]);
}

pub fn fill(id: &str, args: &Args) -> String {
    ready();
    if let Ok(body) = aurorix_template::fill_for(crate::PLUGIN_ID, id, args) {
        return body;
    }
    let Some((_, short)) = id.rsplit_once('/') else {
        return String::new();
    };
    aurorix_template::fill_for(crate::PLUGIN_ID, short, args).unwrap_or_default()
}

fn body_outgoing(body: String) -> Outgoing {
    let mut outgoing = Outgoing::new(ReplyKind::Handler, String::new());
    outgoing.parts = vec![Part::Markdown(body)];
    outgoing
}

pub fn text(id: &str, args: &Args) -> Outgoing {
    hints::finish(id, body_outgoing(fill(id, args)))
}

pub fn filled(id: &str, fallback: &str, args: &Args) -> Outgoing {
    let body = fill(id, args);
    let outgoing = if body.trim().is_empty() {
        Outgoing::new(ReplyKind::Handler, fallback)
    } else {
        body_outgoing(body)
    };
    hints::finish(id, outgoing)
}

pub fn plain(text: impl Into<String>) -> Outgoing {
    hints::finish("", Outgoing::new(ReplyKind::Handler, text))
}

pub fn bare() -> Outgoing {
    hints::finish("", Outgoing::new(ReplyKind::Handler, String::new()))
}

pub fn board(outgoing: Outgoing, keyboard: Option<aurorix_buttons::Keyboard>) -> Outgoing {
    let outgoing = hints::land(outgoing, keyboard.as_ref());
    match keyboard {
        Some(keyboard) => outgoing.keyboard(keyboard),
        None => outgoing,
    }
}

pub fn notice(value: impl Into<String>) -> Outgoing {
    let mut values = args();
    put(&mut values, "reason", value);
    text("shared/notice", &values)
}

pub fn fail(value: impl Into<String>) -> Outgoing {
    let mut values = args();
    put(&mut values, "reason", value);
    text("shared/fail", &values)
}

pub fn ack(heading: impl Into<String>, hint: impl Into<String>) -> Outgoing {
    let mut values = args();
    put(&mut values, "heading", heading);
    put(&mut values, "hint", hint);
    text("shared/ack", &values)
}

pub fn unbound(region: Region) -> Outgoing {
    let key = match region.is_cn() {
        true => "account/account.cn.unbound",
        false => "shared/unbound",
    };
    let values = args();
    board(text(key, &values), buttons::bind_board(region))
}

pub fn pick(root: &str, action: &str, query: &str, found: &SongMatch) -> Outgoing {
    let mut values = args();
    put(&mut values, "query", clip(query, 24));
    let mut all: Vec<&SongHit> = Vec::with_capacity(1 + found.alternatives.len());
    all.push(&found.best);
    all.extend(found.alternatives.iter());
    put(&mut values, "total", found.total.max(all.len()).to_string());
    for (index, hit) in all.iter().enumerate() {
        row(
            &mut values,
            "rows",
            &[
                ("index", (index + 1).to_string()),
                ("label", pick_label(&hit.song)),
                (
                    "command",
                    format!("{root} {action} {}", pick_key(&hit.song, &all)),
                ),
            ],
        );
    }
    text("shared/pick", &values)
}

fn pick_label(song: &Song) -> String {
    if song.artist.trim().is_empty() {
        return song.title.clone();
    }
    format!("{} · {}", song.title, song.artist)
}

fn pick_key(song: &Song, all: &[&SongHit]) -> String {
    let title = song.title.trim();
    if title.is_empty() {
        return song.id.clone();
    }
    let clash = all
        .iter()
        .any(|hit| hit.song.id != song.id && hit.song.title.trim().eq_ignore_ascii_case(title));
    if clash {
        return song.id.clone();
    }
    title.to_string()
}

fn clip(text: &str, limit: usize) -> String {
    let trimmed = text.trim();
    if trimmed.chars().count() <= limit {
        return trimmed.to_string();
    }
    let mut out: String = trimmed.chars().take(limit).collect();
    out.push('…');
    out
}

pub fn card(outgoing: Outgoing, card: RenderCard) -> Outgoing {
    hints::relocate(outgoing.card(card))
}

pub fn card_or(outgoing: Outgoing, card: RenderCard, fallback: String) -> Outgoing {
    hints::relocate(outgoing.card_or(card, fallback))
}

pub fn image(outgoing: Outgoing, url: impl Into<String>, summary: impl Into<String>) -> Outgoing {
    hints::relocate(outgoing.image_url(url, summary))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Chart, MatchKind};
    use crate::model::Difficulty;

    fn hit(id: &str, title: &str, artist: &str) -> SongHit {
        SongHit {
            song: Song::new(id, title)
                .with_artist(artist)
                .with_chart(Chart::new(Difficulty::Future, "9", 9.0)),
            relevance: 1.0,
            matched_by: MatchKind::Title,
        }
    }

    fn matched(best: SongHit, alternatives: Vec<SongHit>, total: usize) -> SongMatch {
        SongMatch {
            best,
            exact: false,
            alternatives,
            total,
        }
    }

    #[test]
    fn case_032() {
        let found = matched(
            hit("renegade", "Renegade", "Aoi"),
            vec![hit("renegade2", "Renegade", "t+pazolite")],
            2,
        );
        let body = pick("/a", "score", "renegade", &found).body();
        assert!(body.contains("对上 2 首"), "要报出对上几首");
        assert_eq!(body.matches("qqbot-cmd-input").count(), 2, "一行一首可点");
        assert!(body.contains(r#"text="%2Fa%20score%20renegade""#));
        assert!(body.contains(r#"text="%2Fa%20score%20renegade2""#));
        assert!(
            body.contains(r#"show="Renegade%20%C2%B7%20Aoi""#),
            "曲师用来区分同名曲"
        );
    }

    #[test]
    fn case_033() {
        let found = matched(hit("ifi", "Infinite Strife,", "Feryquitous"), Vec::new(), 1);
        let body = pick("/ca", "song", "ifi", &found).body();
        assert!(
            body.contains(r#"text="%2Fca%20song%20Infinite%20Strife%2C""#),
            "曲名唯一时用曲名，玩家认不出曲目 ID"
        );
    }

    #[test]
    fn case_034() {
        let long = "一".repeat(40);
        assert_eq!(clip(&long, 24).chars().count(), 25);
        assert!(clip(&long, 24).ends_with('…'));
        assert_eq!(clip(" 短 ", 24), "短");
    }

    fn body(outgoing: &Outgoing) -> String {
        let mut text = String::new();
        for part in &outgoing.parts {
            match part {
                Part::Markdown(value) | Part::Text(value) => text.push_str(value),
                _ => {}
            }
        }
        text
    }

    #[test]
    fn unbound_page_carries_bind_button_per_region() {
        let intl = unbound(Region::Intl);
        assert!(
            intl.keyboard.is_some(),
            "国际服未绑定页要挂「绑定账号」按钮"
        );
        let intl_body = body(&intl);
        assert!(
            intl_body.contains("绑定账号") && intl_body.contains("/a bind"),
            "国际服引导语指向好友码绑定：{intl_body}"
        );
        assert!(
            !intl_body.contains("CN账号"),
            "国际服不该看到国服引导：{intl_body}"
        );

        let cn = unbound(Region::Cn);
        assert!(cn.keyboard.is_some(), "国服未绑定页要挂「登录账号」按钮");
        let cn_body = body(&cn);
        assert!(
            cn_body.contains("CN账号") && cn_body.contains("/ca login"),
            "国服走设备码授权那一页：{cn_body}"
        );
    }
}
