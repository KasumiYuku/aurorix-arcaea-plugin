
use std::fs;

use aurorix_arcaea_plugin::cards::avatar::{self, Avatar};
use aurorix_arcaea_plugin::cards::harness::{self, Harness};
use aurorix_arcaea_plugin::cards::widgets;
use aurorix_render::Color;

const 图标: &str = "char_icons/阿米娅_amiya_102_icon.png";
const 背景: Color = Color::new(32, 30, 44, 255);

#[test]
#[ignore]
fn avatar_overview() {
    let harness = Harness::new();
    let width = 1500;
    let height = 900;
    let bytes = harness.render(harness::request(width, height), |kit| {
        kit.fill_rect(widgets::rect(0.0, 0.0, width as f32, height as f32), 背景);

        let radii = [34.0f32, 48.0, 64.0, 88.0];
        let rows: [(Option<f64>, Option<i32>, Option<i32>); 4] = [
            (None, None, None),
            (Some(9.87), Some(2), None),
            (Some(12.34), Some(4), Some(22)),
            (Some(14.50), Some(8), Some(3)),
        ];

        for (ri, (ptt, frame, rank)) in rows.iter().enumerate() {
            for (ci, radius) in radii.iter().enumerate() {
                let cx = 70.0 + ci as f32 * 350.0 + radius;
                let cy = 70.0 + ri as f32 * 210.0 + radius;
                let spec = Avatar::new(Some(图标))
                    .with_ptt(*ptt)
                    .with_frame(*frame)
                    .with_global_rank(*rank);
                avatar::draw(kit, cx, cy, *radius, &spec);
            }
        }
    });

    let dir = std::path::PathBuf::from("/tmp/arcaea_picture");
    fs::create_dir_all(&dir).expect("样式图目录必须可创建");
    fs::write(dir.join("avatar_style.png"), &bytes).expect("样式图必须可写");
    println!("avatar_style.png -> {} 字节", bytes.len());
}

fn render_avatar(rank: Option<i32>) -> (harness::Pixels, f32, f32) {
    let harness = Harness::new();
    let (width, height) = (520i32, 420i32);
    let (cx, cy, radius) = (180.0f32, 250.0f32, 64.0f32);
    let spec = Avatar::new(Some(图标)).with_global_rank(rank);
    let bytes = harness.render(harness::request(width, height), |kit| {
        kit.fill_rect(widgets::rect(0.0, 0.0, width as f32, height as f32), 背景);
        avatar::draw(kit, cx, cy, radius, &spec);
    });
    let pixels = harness::decode(&bytes).expect("样式图必须能解码");
    (pixels, cx, cy)
}

#[test]
fn ranked_avatar() {
    let (pixels, cx, cy) = render_avatar(Some(233));
    let bg = [32u8, 30, 44, 255];
    let busy = |x: u32, y: u32| pixels.at(x, y) != bg;
    let (wx, wy) = ((cx + 70.0) as u32, (cy - 40.0) as u32);
    assert!(
        busy(wx, wy),
        "头像右上方应是底板自带的排名翼，实测 {:?}",
        pixels.at(wx, wy)
    );
    assert!(
        !busy((cx + 200.0) as u32, wy),
        "翼尖以外不该有内容，实测 {:?}",
        pixels.at((cx + 200.0) as u32, wy)
    );
}

#[test]
fn unranked_avatar() {
    let (pixels, cx, cy) = render_avatar(None);
    let bg = [32u8, 30, 44, 255];
    let (wx, wy) = ((cx + 120.0) as u32, (cy - 40.0) as u32);
    assert!(
        pixels.at(wx, wy) == bg,
        "没排名时头像右上方应该是空的，实测 {:?}",
        pixels.at(wx, wy)
    );
    let (lx, ly) = ((cx - 96.0) as u32, cy as u32);
    assert!(
        pixels.at(lx, ly) == bg,
        "没排名时不该有底板菱形的左尖，实测 {:?}",
        pixels.at(lx, ly)
    );
}

type StyleCase = (Option<f64>, Option<i32>, Option<i32>, &'static str);

#[test]
#[ignore]
fn output_dir_writable() {
    let cases: [StyleCase; 5] = [
        (None, None, None, "no_rank"),
        (Some(9.87), Some(2), None, "ptt_only"),
        (Some(12.34), Some(4), Some(22), "rank_22"),
        (Some(12.34), Some(4), Some(233), "rank_233"),
        (Some(14.50), Some(8), Some(1234567), "rank_wide"),
    ];
    let (width, height) = (720i32, 720i32);
    let (cx, cy, radius) = (360.0f32, 360.0f32, 88.0f32);
    let dir = std::path::PathBuf::from("/tmp/arcaea_picture");
    fs::create_dir_all(&dir).expect("样式图目录必须可创建");
    for (ptt, frame, rank, name) in cases {
        let harness = Harness::new();
        let spec = Avatar::new(Some(图标))
            .with_ptt(ptt)
            .with_frame(frame)
            .with_global_rank(rank);
        let bytes = harness.render(harness::request(width, height), |kit| {
            kit.fill_rect(widgets::rect(0.0, 0.0, width as f32, height as f32), 背景);
            avatar::draw(kit, cx, cy, radius, &spec);
        });
        fs::write(dir.join(format!("avatar_single_{name}.png")), &bytes)
            .expect("输出目录必须可写");
        println!("avatar_single_{name}.png -> {} 字节", bytes.len());
    }
}

fn render_frame(ptt: Option<f64>, frame: Option<i32>) -> harness::Pixels {
    let harness = Harness::new();
    let (width, height) = (420i32, 420i32);
    let (cx, cy, radius) = (210.0f32, 210.0f32, 88.0f32);
    let spec = Avatar::new(Some(图标)).with_ptt(ptt).with_frame(frame);
    let bytes = harness.render(harness::request(width, height), |kit| {
        kit.fill_rect(widgets::rect(0.0, 0.0, width as f32, height as f32), 背景);
        avatar::draw(kit, cx, cy, radius, &spec);
    });
    harness::decode(&bytes).expect("档位样张必须能解码")
}

#[test]
fn frame_tier_changes_badge() {
    let off = render_frame(None, None);
    let five = render_frame(Some(12.34), Some(5));
    let eight = render_frame(Some(12.34), Some(8));
    assert_ne!(off.data, five.data, "档位 5 的头像徽章不该是 rating_off");
    assert_ne!(five.data, eight.data, "档位 5 与档位 8 的徽章必须不同");
}

#[test]
fn missing_frame_derives_from_ptt() {
    let derived = render_frame(Some(12.34), None);
    let explicit = render_frame(Some(12.34), Some(5));
    assert_eq!(
        derived.data, explicit.data,
        "PTT 已知但档位缺失时必须按 PTT 画出同一个徽章, 不能退成 rating_off"
    );
    let off = render_frame(None, None);
    assert_ne!(derived.data, off.data, "有 PTT 与没 PTT 必须画得不一样");
}
