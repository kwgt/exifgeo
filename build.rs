/*
 * Reverse geocoder for Exif location data
 *
 *  Copyright (C) 2025 Hiroshi KUWAGATA <kgt9221@gmail.com>
 */

use std::process::Command;

///
/// ビルドスクリプトのエントリポイント
///
fn main() {
    /*
     * コミットハッシュ埋め込み用の環境変数設定
     */
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_hash());
}

///
/// コミットハッシュ取得関数
///
/// # 戻り値
/// ショートフォーマットのコミットハッシュ文字列
///
fn git_hash() -> String {
    match Command::new("git").args(&["rev-parse", "--short", "HEAD"]).output() {
        Ok(output) => {
            let hash = output.stdout;
            eprintln!("come {}:{}", file!(), line!());

            if hash.is_empty() {
                String::from("unknown")

            } else {
                String::from_utf8(hash)
                    .expect("Git output is not valid UTF-8")
                    .trim()
                    .to_string()
            }
        }

        Err(_) => {
            String::from("unknown")
        }
    }
}
