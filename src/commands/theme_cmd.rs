use aurorix_plugin::{Outgoing, PluginError};

use crate::cards::gallery::{ThemeEntry, ThemeInput};
use crate::commands::{bridge, prefs_of, reply, repo_of, root_of, Ctx};
use crate::model::Prefs;

pub const DEFAULT_LABEL: &str = "默认初始主题";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeOption {
    pub name: String,
    pub label: String,
}

impl ThemeOption {
    pub fn display(&self) -> &str {
        if self.label.is_empty() {
            self.name.as_str()
        } else {
            self.label.as_str()
        }
    }
}

pub fn theme_options() -> Vec<ThemeOption> {
    let mut options = vec![ThemeOption {
        name: String::new(),
        label: DEFAULT_LABEL.to_string(),
    }];
    for theme in crate::cards::theme::Theme::all() {
        options.push(ThemeOption {
            name: theme.name.to_string(),
            label: theme.label.to_string(),
        });
    }
    options
}

fn preview_of(name: &str) -> String {
    crate::cards::theme::Theme::by_name(name)
        .backdrop
        .to_string()
}

pub fn resolve_choice(arg: &str, options: &[ThemeOption]) -> Option<usize> {
    let wanted = arg.trim();
    if wanted.is_empty() {
        return None;
    }
    let lowered = wanted.to_lowercase();
    if matches!(lowered.as_str(), "0" | "default" | "默认" | "原版" | "官方") {
        return Some(0);
    }
    if let Ok(number) = wanted.parse::<usize>() {
        return (1..=options.len())
            .contains(&number)
            .then(|| number - 1);
    }
    options.iter().position(|option| {
        option.name.to_lowercase() == lowered || option.label.to_lowercase() == lowered
    })
}

pub fn option_list(options: &[ThemeOption]) -> String {
    let mut text = format!("可用主题 {} 个：", options.len());
    for (index, option) in options.iter().enumerate() {
        if option.name.is_empty() {
            text.push_str(&format!(
                "\n  {}. {}（写 0 或 default 也行）",
                index + 1,
                option.display()
            ));
        } else {
            text.push_str(&format!(
                "\n  {}. {}（{}）",
                index + 1,
                option.display(),
                option.name
            ));
        }
    }
    text
}

pub async fn theme(ctx: &Ctx<'_>, arg: &str) -> Result<Option<Outgoing>, PluginError> {
    let options = theme_options();
    let repo = repo_of(ctx).await.ok();
    let mut prefs = match repo.as_ref() {
        Some(repo) => prefs_of(ctx, repo).await,
        None => Prefs::default(),
    };
    let mut hint = format!(
        "发 {} theme <编号或名字> 切换出图主题",
        root_of(ctx)
    );
    let wanted = arg.trim();
    if !wanted.is_empty() {
        let Some(index) = resolve_choice(wanted, &options) else {
            return Ok(Some(reply::notice(format!(
                "没有名为「{wanted}」的主题\n\n{}",
                option_list(&options)
            ))));
        };
        let Some(repo) = repo else {
            return Ok(Some(reply::fail("插件数据库还没准备好")));
        };
        let option = &options[index];
        let mut next = prefs.clone();
        next.theme = option.name.clone();
        if let Err(error) = repo.set_prefs(ctx.region, &ctx.inv.user, &next).await {
            return Ok(Some(reply::fail(error.to_string())));
        }
        hint = if option.name.is_empty() {
            format!("已切回{}", option.display())
        } else {
            format!("已切换到 {}（{}）", option.display(), option.name)
        };
        prefs = next;
    }
    Ok(Some(page(&options, &prefs, hint.as_str())))
}

fn page(options: &[ThemeOption], prefs: &Prefs, hint: &str) -> Outgoing {
    let theme = bridge::theme_of(prefs);
    let entries: Vec<ThemeEntry> = options
        .iter()
        .map(|option| {
            ThemeEntry::new(option.name.clone(), option.label.clone())
                .with_preview(preview_of(option.name.as_str()))
                .with_active(option.name == prefs.theme)
        })
        .collect();
    let input = ThemeInput::new()
        .with_theme(theme)
        .with_note(hint.to_string())
        .with_entries(entries);
    let mut args = reply::args();
    reply::put(&mut args, "hint", hint);
    reply::card(
        reply::text("theme/theme.switch", &args),
        crate::cards::gallery::theme_card(input),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_010() {
        let options = theme_options();
        assert_eq!(options[0].name, "", "默认主题的名字必须是空串");
        assert_eq!(options[0].label, DEFAULT_LABEL);
        assert!(options.len() >= 2, "素材里至少有一个主题目录");
        assert!(
            options[1..]
                .windows(2)
                .all(|pair| pair[0].name <= pair[1].name),
            "主题目录必须按名字排序"
        );
        assert!(
            options[1..].iter().all(|option| !option.label.is_empty()),
            "标签读不到时也要回落到目录名"
        );
    }

    #[test]
    fn case_011() {
        let options = theme_options();
        for text in ["0", "default", "DEFAULT", "默认", "原版", "官方", " 0 "] {
            assert_eq!(
                resolve_choice(text, &options),
                Some(0),
                "{text} 必须解析成默认主题"
            );
        }
    }

    #[test]
    fn case_012() {
        let options = theme_options();
        let index = options
            .iter()
            .position(|option| option.name == "arknights")
            .expect("素材里必须有 arknights 主题");
        assert_eq!(resolve_choice(&(index + 1).to_string(), &options), Some(index));
        assert_eq!(resolve_choice("arknights", &options), Some(index));
        assert_eq!(resolve_choice("Arknights", &options), Some(index));
        assert_eq!(resolve_choice(" ARKNIGHTS ", &options), Some(index));
        assert_eq!(resolve_choice("明日方舟", &options), None, "中文名不是标签");
        assert_eq!(resolve_choice("glacier", &options), None);
    }

    #[test]
    fn case_013() {
        let options = theme_options();
        let too_big = (options.len() + 1).to_string();
        assert_eq!(resolve_choice(&too_big, &options), None);
        assert_eq!(resolve_choice("", &options), None);
    }

    #[test]
    fn case_014() {
        let options = theme_options();
        let text = option_list(&options);
        assert!(text.contains(DEFAULT_LABEL));
        for option in &options[1..] {
            assert!(text.contains(option.name.as_str()), "{} 没列进去", option.name);
        }
    }

    #[test]
    fn case_015() {
        let options = theme_options();
        let entries: Vec<ThemeEntry> = options
            .iter()
            .map(|option| ThemeEntry::new(option.name.clone(), option.label.clone()))
            .collect();
        let input = ThemeInput::new().with_entries(entries);
        let (width, height) = crate::cards::gallery::theme_size(&input);
        let spec = crate::cards::spec_of(crate::cards::CardKind::Theme);
        assert!(spec.accepts(width, height), "{width}x{height} 不合规格");
    }
}
