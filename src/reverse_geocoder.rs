/*
 * Reverse geocoder for Exif location data
 *
 *  Copyright (C) 2025 Hiroshi KUWAGATA <kgt9221@gmail.com>
 */

//!
//! 国土地理院逆ジオコーディング APIの呼び出しによる緯度経度→住所変換処理をま
//! とめたモジュール
//!

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::Deserialize;

use crate::cmd_args::Options;
use crate::municd::{self, MuniCdRecord};

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

/// 国土地理院逆ジオコーディングAPIのベースURL
const REVERSE_GEOCODE_URL: &str =
      "https://mreversegeocoder.gsi.go.jp/reverse-geocoder/LonLatToAddress";

///
/// APIからのレスポンスを格納する構造体
///
#[derive(Debug, Deserialize)]
struct ApiResult {
    /// 位置情報
    results: GeometoryInfo,
}

impl ApiResult {
    ///
    /// 市町村コードへのアクセサ
    ///
    /// # 戻り値
    /// 正規化された文字列に変換された市町村コードを返す
    ///
    fn muni_code(&self) -> String {
        format!("{:0>6}", self.results.muni_code)
    }

    ///
    /// 地域名へのアクセサ
    ///
    /// # 戻り値
    /// 地域名を文字列で返す
    ///
    /// # 注記
    /// 本メソッドで返される地域名が含まれる市町村はMuniCdデータベースから取得
    /// できる。MuniCdデータベースから`ApiResult::muni_code()`で返される市町村
    /// コードで検索したエントリから得ることが出来る。
    ///
    fn area_name(&self) -> String {
        self.results.area_name.clone()
    }
 }

///
/// 位置情報を格納した構造体
///
#[derive(Debug, Deserialize)]
struct GeometoryInfo {
    /// 位置情報に対応する市町村コード
    #[serde(rename="muniCd")]
    muni_code: String,

    /// 地域名
    #[serde(rename="lv01Nm")]
    area_name: String,
}

///
/// 逆ジオコーディングインタフェース構造体
///
#[derive(Debug)]
pub(crate) struct ReverseGeocoder {
    /// 市町村コードをキーとした市町村データベース
    municd: HashMap<String, MuniCdRecord>,
}

impl ReverseGeocoder {
    ///
    /// 逆ジオコーディングインターフェースオブジェクトの生成
    ///
    /// # 引数
    /// * `opts` - オプション情報をパックしたオブジェクト
    ///
    /// # 戻り値
    /// 処理に成功した場合は、生成したオブジェクトを`Ok()`でラップして返す。
    ///
    pub(crate) fn new(opts: Arc<Options>) -> Result<Self> {
        Ok(Self { municd: municd::load(opts.municd_cache())?})
    }

    ///
    /// 住所の照会
    ///
    /// # 引数
    /// * `lat` - 照会する北緯 
    /// * `lng` - 照会する東経
    ///
    /// # 戻り値
    /// 照会に成功した場合は、照会できた住所を`Ok()`でラップして返す。
    ///
    /// # 注記
    /// 本メソッドは、国土地理院の逆ジオコーディングAPIの呼び出しを行う。
    ///
    pub(crate) fn query(&self, lat: f64, lng: f64) -> Result<String> {
        let url = format!("{}?lat={}&lon={}", REVERSE_GEOCODE_URL, lat, lng);

        info!("query to {}", url);
        let result = reqwest::blocking::get(url)?.json::<ApiResult>()?;

        debug!("{:?}", result);

        match self.municd.get(&result.muni_code()) {
            Some(muni) => {
                Ok(format!("{}{}", muni.to_string(), result.area_name()))
            }

            None => {
                Ok(format!("????? {}", result.area_name()))
            }
        }
    }
}
