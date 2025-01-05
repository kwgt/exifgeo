/*
 * Reverse geometory for Exif location data
 *
 *  Copyright (C) 2025 Hiroshi KUWAGATA <kgt9221@gmail.com>
 */

//!
//! コマンドラインオプション関連の処理をまとめたモジュール
//!

mod logger;

use std::sync::Arc;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Parser, ValueEnum};

///
/// ログレベルを指し示す列挙子
///
#[derive(Debug, Clone, Copy, PartialEq, ValueEnum)]
#[clap(rename_all = "SCREAMING_SNAKE_CASE")]
enum LogLevel {
    /// ログを記録しない
    Off,

    /// エラー情報以上のレベルを記録
    Error,

    /// 警告情報以上のレベルを記録
    Warn,

    /// 一般情報以上のレベルを記録
    Info,

    /// デバッグ情報以上のレベルを記録
    Debug,

    /// トレース情報以上のレベルを記録
    Trace,
}

// AsRefトレイトの実装
impl AsRef<str> for LogLevel {
    fn as_ref(&self) -> &str {
        match self {
            Self::Off => "none",
            Self::Error => "error",
            Self::Warn => "warn",
            Self::Info => "info",
            Self::Debug => "debug",
            Self::Trace => "trace",
        }
    }
}

// ToStringトレイトの実装
impl ToString for LogLevel {
    fn to_string(&self) -> String {
        self.as_ref().to_string()
    }
}

///
/// コマンドラインオプションをまとめた構造体
///
#[derive(Parser, Debug, Clone)]
#[command(about = "Reverse geometory for Exif location data")]
#[command(version = concat!(
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_COMMIT_HASH"),
    ")",
))]
#[command(long_about = None)]
pub(crate) struct Options {
    /// 記録するログレベルの指定
    #[arg(short = 'l', long = "log-level", value_name = "LEVEL",
        default_value = "OFF", ignore_case = true)]
    log_level: LogLevel,

    /// 市町村コードのキャッシュファイルへのパス
    #[arg(short = 'm', long = "municd-cache", value_name = "FILE",
        default_value_t = default_municd_cache())]
    municd_cache: String,

    /// 処理対象のファイル名
    #[arg()]
    target_files: Vec<PathBuf>,
}

impl Options {
    ///
    /// ログレベルへのアクセサ
    ///
    /// # 戻り値
    /// 設定されたログレベルを返す
    ///
    fn log_level(&self) -> LogLevel {
        self.log_level
    }

    ///
    /// 住所コードキャッシュファイルのパスへのアクセサ
    ///
    /// # 戻り値
    /// 住所コードキャッシュファイルへのパス情報
    ///
    pub(crate) fn municd_cache(&self) -> PathBuf {
        PathBuf::from(self.municd_cache.clone())
    }

    ///
    /// 処理対象ファイルへのアクセサ
    ///
    /// # 戻り値
    /// コマンドラインで指定されたファイルのリスト
    ///
    pub(crate) fn target_files(&self) -> &Vec<PathBuf> {
        &self.target_files
    }

    ///
    /// 設定情報のバリデーション
    ///
    /// # 戻り値
    /// 設定情報に問題が無い場合は`Ok(())`を返す。問題があった場合はエラー情報
    /// を`Err()`でラップして返す。
    ///
    fn validate(&self) -> Result<()> {
        // 指定ファイルのチェック
        if self.target_files.is_empty() {
            return Err(anyhow!("target files is not specified"));
        }

        Ok(())
    }
}

///
/// 住所コードキャッシュファイルへのパスへのデフォルト値
///
fn default_municd_cache() -> String {
    if let Some(prefix) = dirs_next::cache_dir() {
        prefix
            .join(env!("CARGO_PKG_NAME"))
            .join("municd.json")
            .display()
            .to_string()
    } else {
        String::from("municd.json")
    }
}

///
/// コマンドラインオプションのパース
///
/// # 戻り値
/// 処理に成功した場合はオプション設定をパックしたオブジェクトを`Ok()`でラップ
/// して返す。失敗した場合はエラー情報を`Err()`でラップして返す。
///
pub(super) fn parse() -> Result<Arc<Options>> {
    let opts = Options::parse();

    /*
     * 設定情報のバリデーション
     */
    opts.validate()?;

    /*
     * ログ機能の初期化
     */
    logger::init(&opts)?;

    /*
     * 設定情報の返却
     */
    Ok(Arc::new(opts))
}
