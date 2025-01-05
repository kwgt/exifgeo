/*
 * Reverse geocoder for Exif location data
 *
 *  Copyright (C) 2025 Hiroshi KUWAGATA <kgt9221@gmail.com>
 */

//!
//! プログラムのエントリポイント
//!

mod cmd_args;
mod gps_info;
mod municd;
mod reverse_geocoder;

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;

use cmd_args::Options;
use reverse_geocoder::ReverseGeocoder;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

///
/// プログラムのエントリポイント
///
fn main() {
    let opts = match cmd_args::parse() {
        Ok(opts) => opts,
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    };

    if let Err(err) = run(opts) {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

///
/// プログラムの実行関数
///
/// # 引数
/// * `opts` - オプション情報をパックしたオブジェクト
///
/// # 戻り値
/// 処理に成功した場合は`Ok(())`を返す。失敗した場合はエラー情報を`Err()` でラ
/// ップして返す。
///
fn run(opts: Arc<Options>) -> Result<()> {
    /*
     * 逆ジオコーディングインタフェースの生成
     */
    let coder = ReverseGeocoder::new(opts.clone())?;

    /*
     * ファイル毎に処理を実施
     */
    for file in opts.target_files() {
        info!("try {}", file.display());

        // 位置情報の読み出し
        let (lat, lng) = match gps_info::read(&file) {
            Ok(Some((lat, lng))) => (lat, lng), 

            Ok(None) => {
                gps_info_none(&file);
                continue;
            }

            Err(err) => {
                read_failed(&file, err);
                continue;
            }
        };

        // 住所の問い合わせ(逆ジオコーディングAPIの呼び出し)
        match coder.query(lat, lng) {
            Ok(address) => query_succeed(&file, address, lat, lng),
            Err(err) => query_failed(&file, err, lat, lng),
        }
    }

    /*
     * 終了
     */
    Ok(())
}

///
/// 処理に成功し住所を取得できた場合の表示関数
///
/// # 引数
/// * `file` - 処理対象のファイルのパス情報
/// * `address` - 住所
/// * `lat` - 北緯
/// * `lng` - 東経
///
fn query_succeed(file: &PathBuf, address: String, lat: f64, lng: f64) {
    println!(
        "{}\n\t{} ({:.2}\u{00b0},{:.2}\u{00b0})",
        file.file_name().unwrap().to_str().unwrap(),
        address,
        lat,
        lng
    );
}

///
/// 住所の取得に失敗した場合の表示関数
///
/// # 引数
/// * `file` - 処理対象のファイルのパス情報
/// * `err` - エラー情報
/// * `lat` - 北緯
/// * `lng` - 東経
///
fn query_failed(file: &PathBuf, err: anyhow::Error, lat: f64, lng: f64) {
    eprintln!(
        "{}: 位置情報問い合わせ失敗({:.2}\u{00b0},{:.2}\u{00b0}, {})",
        file.display(),
        lat,
        lng,
        err
    );
}

///
/// ファイルに位置情報が含まれていなかった場合の表示関数
///  
/// # 引数
/// * `file` - 処理対象のファイルのパス情報
///
fn gps_info_none(file: &PathBuf) {
    eprintln!("{}: 位置情報無し", file.display());
}

///
/// 位置情報の読み出しに失敗した場合の表示関数
///
/// # 引数
/// * `file` - 処理対象のファイルのパス情報
/// * `err` - エラー情報
///
fn read_failed(file: &PathBuf, err: anyhow::Error) {
    eprintln!("{}: 位置情報読み出し失敗({})", file.display(), err);
}
