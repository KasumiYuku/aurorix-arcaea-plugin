use std::path::{Path, PathBuf};

use aurorix_plugin::PluginAssets;

pub mod backend;
pub mod commands;
pub mod cards;
pub mod domain;
pub mod model;
pub mod repo;
pub mod templates;
pub mod warm;

pub use model::{Account, Difficulty, Grade, Prefs, Region, Skin, REGIONS};
pub use templates::{register_templates, template_ids, MARKDOWN};
pub use warm::{warmup, Warmup, CORE_TEXTURES};

pub const PLUGIN_ID: &str = "arcaea";

pub const ASSETS_DIR: &str = "assets";

pub const FONT_FILES: [&str; 4] = [
    "fonts/NotoSans-Bold.ttf",
    "fonts/NotoSans-Medium.ttf",
    "fonts/L2-Regular.ttf",
    "fonts/Exo-Semibold.ttf",
];

pub const IMAGE_FILES: [&str; 2] = ["logo/arcaea_logo.png", "difficulty/max-recall-future.png"];

pub const MEDIA_DIRS: [&str; 5] = [
    "jackets",
    "jacket_thumbs",
    "characters",
    "char_icons",
    "background",
];

pub fn assets_root() -> PathBuf {
    let local = Path::new(ASSETS_DIR).join(PLUGIN_ID);
    if local.is_dir() {
        return local;
    }
    Path::new(env!("CARGO_MANIFEST_DIR")).join(ASSETS_DIR)
}

pub fn assets() -> PluginAssets {
    assets_in(assets_root())
}

pub fn assets_in(root: impl Into<PathBuf>) -> PluginAssets {
    PluginAssets::new(PLUGIN_ID, root)
}

pub async fn install(
    db: std::sync::Arc<aurorix_db::PluginDb>,
    backend: std::sync::Arc<crate::backend::ArcBackendNext>,
) -> Result<aurorix_plugin::PluginCommands, String> {
    install_with_report(db, backend)
        .await
        .map(|(commands, _)| commands)
}

pub async fn install_with_report(
    db: std::sync::Arc<aurorix_db::PluginDb>,
    backend: std::sync::Arc<crate::backend::ArcBackendNext>,
) -> Result<(aurorix_plugin::PluginCommands, String), String> {
    crate::repo::ArcaeaRepo::open(db.clone())
        .await
        .map_err(|error| format!("插件数据库迁移失败：{error}"))?;
    let mut notes: Vec<String> = Vec::new();
    match crate::repo::legacy::import_once(&db).await {
        Ok(report) => notes.push(format!("旧数据迁移 {} 条", report.total())),
        Err(error) => notes.push(format!("旧数据迁移跳过：{error}")),
    }
    let templates = crate::templates::register_templates(aurorix_template::Registry::global());
    notes.push(format!("模板 {templates} 个"));
    let assets = std::sync::Arc::new(assets());
    let context = aurorix_render::RenderContext::new(
        aurorix_render::SkiaBackend::handle(),
        crate::cards::harness::FONT_CAPACITY,
        crate::cards::harness::TEXTURE_CAPACITY,
    )
    .with_asset_loader(assets.loader());
    let warm = crate::warm::warmup(&assets, &context);
    notes.push(format!(
        "预热 字体 {} · 模板 {} · 贴图 {} · 缺失 {}",
        warm.fonts, warm.templates, warm.textures, warm.missing
    ));
    let commands = crate::commands::plugin(crate::commands::Services::new(backend.clone()));
    let tips = crate::commands::hints::install(&commands.commands)
        .map_err(|error| format!("提示行装配失败：{error}"))?;
    notes.push(format!("提示行候选 {tips} 条"));
    let callbacks = crate::commands::buttons::install_callbacks(backend, db);
    notes.push(format!("按钮回调 {callbacks} 个"));
    Ok((commands, notes.join("；")))
}
