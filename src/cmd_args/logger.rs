/*
 * Reverse geocoder for Exif location data
 *
 *  Copyright (C) 2025 Hiroshi KUWAGATA <kgt9221@gmail.com>
 */

//!
//! ロガーの初期化処理をまとめたモジュール
//!

use anyhow::Result;
use super::{LogLevel, Options};

///
/// ロガーの初期化
///
/// # 引数
/// * `opts` - 設定情報をパックしたオブジェクト
///
pub(super) fn init(opts: &Options) -> Result<()> {
    if opts.log_level() != LogLevel::Off {
        std::env::set_var("RUST_LOG", opts.log_level().to_string());
        env_logger::init();
    }

    Ok(())
}
