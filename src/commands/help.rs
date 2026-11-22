use aurorix_core::Scene;
use aurorix_plugin::Outgoing;

use crate::commands::{buttons, reply, Ctx};

pub fn root(ctx: &Ctx<'_>) -> Outgoing {
    let id = if ctx.region.is_cn() { "help.cn" } else { "help" };
    let outgoing = reply::text(id, &reply::args());
    let outgoing = reply::board(outgoing, buttons::help_board(ctx.region));
    match ctx.inv.scene {
        Scene::C2c | Scene::Dm => outgoing,
        Scene::Group | Scene::Channel => outgoing.no_footer(),
    }
}
