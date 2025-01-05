/*
 * Reverse geocoder for Exif location data
 *
 *  Copyright (C) 2025 Hiroshi KUWAGATA <kgt9221@gmail.com>
 */

//!
//! 国土地理院逆ジオコーディング APIで使用するMuniCdデータの取得処理をまとめ
//! たモジュール
//!

use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::{SystemTime, Duration};

use anyhow::{anyhow, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

/// muniCdのテーブル生成コード取得URL
const MUNICD_URL: &str = "https://maps.gsi.go.jp/js/muni.js";

/// muniCdのテーブル生成コードからレコード生成用データを抽出する為の正規表現
const RECORD_RE: &str =
    "GSI\\.MUNI_ARRAY\\[\"\\d+\"\\] = '\\d+,(.+),(\\d+),(.+)'";

/// キャッシュファイルの有効期間(日数)
const EXPIRE_DAYS: u64 = 10;

///
/// muniCdのレコード
///
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MuniCdRecord {
    /// 市町村コード
    code: usize,

    /// 県名
    pref: String,

    /// 市町村名
    town: String,
}

impl MuniCdRecord {
    ///
    /// オブジェクトの生成
    ///
    /// # 引数
    /// * `code` - 市町村コード
    /// * `pref` - 都道府県名
    /// * `town` - 市町村名
    ///
    /// # 戻り値
    /// 生成されたオブジェクトを返す。
    ///
    fn new(code: usize, pref: String, town: String) -> Self {
        Self {code, pref, town}
    }

    ///
    /// 市町村コードへのアクセサ
    ///
    #[allow(dead_code)]
    pub(crate) fn code(&self) -> usize {
        self.code
    }

    ///
    /// 都道府県名へのアクセサ
    ///
    #[allow(dead_code)]
    pub(crate) fn pref_name(&self) -> String {
        self.pref.clone()
    }

    ///
    /// 市町村名へのアクセサ
    ///
    #[allow(dead_code)]
    pub(crate) fn town_name(&self) -> String {
        self.town.clone()
    }
}

// ToStringトレイトの実装
impl ToString for MuniCdRecord {
    fn to_string(&self) -> String {
        format!("{}{}", self.pref, self.town.replace('\u{3000}', ""))
    }
}

///
/// 市町村データの読み込み
///
/// # 引数
/// * `cache_path` - キャッシュファイルへのパス
///
/// # 戻り値
/// 処理に成功した場合、市町村コードをキーとしたMuniCDレコードのハッシュマップ
/// オブジェクトを`Ok()`でラップして返す。
///
/// # 注記
/// まず、キャッシュファイルからの読み込みを試みて失敗 (もしくはキャッシュファ
/// イルが)無効な場合、サーバからダウンロードしキャッシュを更新する。
///
pub(crate) fn load(cache_path: impl AsRef<Path>)
    -> Result<HashMap<String, MuniCdRecord>>
{
    match load_from_cache(cache_path.as_ref()) {
        Ok(map) => {
            info!("read muniCd data from cache file.");
            Ok(map)
        }

        Err(err) => {
            error!("{}", err);
            download(cache_path)
        }
    }
}

///
/// キャッシュファイルからの市町村データの読み込み
///
/// # 引数
/// * `cache_path` - キャッシュファイルへのパス
///
/// # 戻り値
/// 処理に成功した場合、市町村コードをキーとしたMuniCDレコードのハッシュマップ
/// オブジェクトを`Ok()`でラップして返す。
///
fn load_from_cache(cache_path: &Path)
    -> Result<HashMap<String, MuniCdRecord>>
{
    /*
     * キャッシュファイルの有効性の確認
     */
    if !is_available(cache_path) {
        return Err(anyhow!("cache file is expired."));
    }

    /*
     * キャッシュファイルのオープン
     */
    let file = match File::open(cache_path) {
        Ok(file) => file,
        Err(err) => return Err(anyhow!("open cache file failed: {}", err)),
    };

    /*
     * キャッシュファイルの読み込み
     */
    match serde_json::from_reader(file) {
        Ok(map) => Ok(map),
        Err(err) => return Err(anyhow!("parse JSON failed: {}", err)),
    }
}

///
/// キャッシュファイルの有効性の確認
///
/// # 引数
/// * `path` - 確認対象のファイルのパス
///
/// # 戻り値
/// ファイルが有効な場合は真を返す
///
fn is_available(path: &Path) -> bool {
    /*
     * ファイルが存在するか否かを確認
     */
    if !path.exists() {
        error!("cache file is not exists.");
        return false;
    }

    /*
     * ファイルの更新から規定の期間を過ぎているか否かの確認
     */

    // ファイルのメタ情報の取得
    let meta = match path.metadata() {
        Ok(meta) => meta,
        Err(err) => {
            error!("metadata read failed: {}", err);
            return false;
        }
    };

    // ファイル更新時刻の取得
    let mtime = match meta.modified() {
        Ok(mtime) => mtime,
        Err(err) => {
            error!("read modified time failed: {}", err);
            return false;
        }
    };

    // 直近の更新からの経過時間評価(規定の日数を過ぎてたら無効)
    match SystemTime::now().duration_since(mtime) {
        Ok(diff) => {
            diff < Duration::from_secs(86400 * EXPIRE_DAYS)
        }

        Err(err) => {
            error!("calc since duration failed: {}", err);
            return false;
        }
    }
}

///
/// MuniCDデータベースのダウンロード
///
/// # 引数
/// * `cache_path` - キャッシュファイルへのパス
///
/// # 戻り値
/// 市町村コードをキーとしたMuniCDレコードのハッシュマップオブジェクトを返す。
///
/// # 注記
/// データベースのダウンロードと同時に、キャッシュファイルへの書き込みを行う。
///
fn download(cache_path: impl AsRef<Path>)
    -> Result<HashMap<String, MuniCdRecord>>
{
    let mut ret = HashMap::new();
    let re = Regex::new(RECORD_RE)?;

    info!("try download muniCd data.");
    for line in reqwest::blocking::get(MUNICD_URL)?.text()?.lines() {
        if let Some(captures) = re.captures(line) {
            let pref = captures[1].to_string();
            let code = captures[2].parse::<usize>()?;
            let town = captures[3].to_string();

            ret.insert(
                format!("{:06}", code),
                MuniCdRecord::new(code, pref, town)
            );
        }
    }

    info!("save muniCd data to cache file.");
    if let Some(dir) = cache_path.as_ref().parent() {
        if !dir.exists() {
            std::fs::create_dir_all(dir)?;
        }
    }

    std::fs::write(cache_path, serde_json::to_string(&ret)?)?;

    Ok(ret)
}
