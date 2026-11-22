use std::fs;

use aurorix_arcaea_plugin::{
    assets, assets_in, assets_root, ASSETS_DIR, FONT_FILES, IMAGE_FILES, MEDIA_DIRS,
};

fn is_font(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x00, 0x01, 0x00, 0x00])
        || bytes.starts_with(b"true")
        || bytes.starts_with(b"OTTO")
        || bytes.starts_with(b"ttcf")
}

#[test]
fn case_431() {
    let root = assets_root();
    assert!(root.is_dir(), "素材根目录要可读: {}", root.display());
    assert_eq!(
        root.file_name().and_then(|name| name.to_str()),
        Some(ASSETS_DIR),
        "素材根目录必须叫 assets"
    );
}

#[test]
fn case_432() {
    let store = assets();
    assert!(store.root_ready(), "素材根目录要能被识别");
    let fonts = store.fonts();
    assert!(!fonts.is_empty(), "fonts 目录要至少有一个字体");
    assert!(
        fonts.iter().any(|alias| alias.contains("NotoSans")),
        "字体别名里应当能看到 NotoSans, 实际: {fonts:?}"
    );
    for key in FONT_FILES {
        let bytes = store
            .read(key)
            .unwrap_or_else(|error| panic!("读取 {key} 失败: {error}"));
        assert!(bytes.len() > 1024, "{key} 只有 {} 字节", bytes.len());
        assert!(is_font(&bytes), "{key} 不是可识别的字体文件");
    }
}

#[test]
fn case_433() {
    let store = assets();
    for key in IMAGE_FILES {
        let bytes = store
            .read(key)
            .unwrap_or_else(|error| panic!("读取 {key} 失败: {error}"));
        assert!(
            bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "{key} 不是 PNG 文件"
        );
    }
    assert!(
        !store.exists("characters"),
        "目录不是文件, exists 对目录应当返回 false"
    );
}

#[test]
fn case_434() {
    let root = assets_root();
    for dir in MEDIA_DIRS {
        let path = root.join(dir);
        assert!(path.is_dir(), "{dir} 目录要存在: {}", path.display());
        let files = fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("列出 {dir} 失败: {error}"))
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_file())
            .count();
        assert!(files > 0, "{dir} 目录不能是空的");
    }
}

#[test]
fn case_435() {
    let store = assets();
    let error = store.read("../Cargo.toml").expect_err("越界路径必须被拒绝");
    let text = error.to_string();
    assert!(
        text.contains("越界") || text.contains("绝对"),
        "越界要报明确错误, 实际: {text}"
    );
}

#[test]
fn case_436() {
    let store = assets_in(ASSETS_DIR);
    assert!(store.root_ready(), "相对根目录也要能识别");
    assert_eq!(store.plugin_id(), "arcaea");
    assert!(!store.fonts().is_empty(), "相对根目录下也要能列出字体");
    assert!(
        store.templates().is_empty(),
        "素材仓库目前不带回复模板, 模板走插件自己的 templates 目录"
    );
}
