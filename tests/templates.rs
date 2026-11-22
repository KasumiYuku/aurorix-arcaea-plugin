use std::collections::BTreeMap;

use aurorix_arcaea_plugin::templates::{register_templates, template_ids, MARKDOWN};
use aurorix_arcaea_plugin::PLUGIN_ID;
use aurorix_template::{Arg, Args, Registry, Template, TemplateError, TemplateKind};

fn case_494() -> Registry {
    let registry = Registry::new();
    let count = register_templates(&registry);
    assert_eq!(count, MARKDOWN.len(), "注册数必须与静态表长度一致");
    registry
}

fn case_495(registry: &Registry, id: &str) -> String {
    registry
        .get(TemplateKind::Markdown, PLUGIN_ID, id)
        .expect("模板必须已注册")
        .body()
        .to_string()
}

fn case_496(template: &Template) -> Args {
    let mut args: Args = BTreeMap::new();
    for name in template.args() {
        args.insert(name.clone(), Arg::Str(String::from("占位")));
    }
    let body = template.body();
    let mut rest = body;
    while let Some(index) = rest.find("{{#each ") {
        let tail = &rest[index + 8..];
        let Some(end) = tail.find("}}") else {
            break;
        };
        let name = tail[..end].trim();
        args.insert(name.to_string(), Arg::List(Vec::new()));
        rest = &tail[end..];
    }
    args
}

#[test]
fn case_497() {
    let registry = case_494();
    assert_eq!(registry.count(TemplateKind::Markdown), MARKDOWN.len());
    assert_eq!(template_ids().len(), 59, "迁入的模板数量必须与旧仓库一致");
    for id in template_ids() {
        assert!(
            registry.get(TemplateKind::Markdown, PLUGIN_ID, id).is_some(),
            "{id} 没有注册到 {PLUGIN_ID} 命名空间"
        );
    }
}

#[test]
fn case_498() {
    let registry = case_494();
    for id in template_ids() {
        let template = registry
            .get(TemplateKind::Markdown, PLUGIN_ID, id)
            .expect("模板必须已注册");
        let args = case_496(&template);
        match registry.fill(TemplateKind::Markdown, PLUGIN_ID, id, &args) {
            Ok(body) => assert!(!body.is_empty(), "{id} 渲染出空正文"),
            Err(TemplateError::InlineEmptyCommand) => {}
            Err(other) => panic!("{id} 渲染失败: {other:?}"),
        }
    }
}

#[test]
fn case_499() {
    let registry = case_494();
    let intl = case_495(&registry, "help");
    assert!(intl.contains("/a table"), "help 必须写进 table 指令");
    assert!(intl.contains("/a ahv"), "help 的 ahv 根指令是 /a");
    let cn = case_495(&registry, "help.cn");
    assert!(cn.contains("/ca login"), "help.cn 必须写进 login");
    assert!(cn.contains("/ca unbind"), "help.cn 必须写进 unbind");
    assert!(cn.contains("/ca ahv"), "help.cn 的 ahv 根指令必须是 /ca");
    assert!(!cn.contains("/a ahv"), "help.cn 不能再留国际服根指令");
}

#[test]
fn case_500() {
    for id in template_ids() {
        assert!(!id.starts_with("update-"), "{id} 属于公告死模板");
        assert!(!id.starts_with("announce-"), "{id} 属于公告死模板");
    }
}
