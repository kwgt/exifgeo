/*
 * Reverse geocoder for Exif location data
 *
 *  Copyright (C) 2025 Hiroshi KUWAGATA <kgt9221@gmail.com>
 */

//!
//! Exifの位置情報取得処理をまとめたモジュール
//!

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::{anyhow, Result};
use exif::{Exif, Tag};

///
/// ExifのGPS情報の読み出し
///
/// # 引数
/// * `path` - 読み出し対象のファイルのパス
///
/// # 戻り値
/// GPS 情報の読み出しに成功した場合は読み出した緯度と経度をパックしたタプルを
/// `Ok(Some())` でラップして返す。対象ファイルにGPS情報が存在しなかった場合は
/// `Ok(None)`を返す。
/// 処理に失敗した場合(対象ファイルが未サポートの形式の場合も含む)はエラー情報 
/// を`Err()`でラップして返す。
///
pub(crate) fn read(path: impl AsRef<Path>) -> Result<Option<(f64, f64)>> {
    /*
     * Exif情報の読み出し
     */
    let exif = read_exif(path)?;

    /*
     * 緯度情報の取得
     */
    let lat = match exif
        .get_field(Tag::GPSLatitude, exif::In::PRIMARY)
    {
        Some(val) => val,
        None => return Ok(None),
    };

    let lat_ref = match exif
        .get_field(Tag::GPSLatitudeRef, exif::In::PRIMARY)
    {
        Some(val) => val,
        None => return Ok(None),
    };

    /*
     * 経度情報の取得
     */
    let lng = match exif
        .get_field(Tag::GPSLongitude, exif::In::PRIMARY)
    {
        Some(val) => val,
        None => return Ok(None),
    };

    let lng_ref = match exif
        .get_field(Tag::GPSLongitudeRef, exif::In::PRIMARY)
    {
        Some(val) => val,
        None => return Ok(None),
    };

    /*
     * 緯度及び経度を有理数表現から浮動小数点数に変換し、戻り値として返却
     */
    Ok(Some((conv_degree(&lat, &lat_ref)?, conv_degree(&lng, &lng_ref)?)))
}

///
/// Exif情報の読み出し
///
/// # 引数
/// * `path` - 読み出し対象のファイルのパス
///
/// # 戻り値
/// 読み出しに成功した場合、読み出したExif情報を`Ok()`でラップして返す。
///
fn read_exif(path: impl AsRef<Path>) -> Result<Exif> {
    let mut bufreader = BufReader::new(File::open(path.as_ref())?);

    match exif::Reader::new().read_from_container(&mut bufreader) {
        Ok(exif_data) => Ok(exif_data),
        Err(err) => Err(anyhow!("read exif failed: {}", err)),
    }
}

///
/// 緯度または経度情報の有理数表現された値から浮動小数点数への変換
///
/// # 引数
/// * `value` - 緯度または経度が格納されたExifフィールド情報
/// * `reference` - 緯度または経度に付随する参照情報が格納されたExifフィールド
///
/// # 戻り値
/// 正常に変換できた場合は、変換された値を`Ok()`でラップして返す。
///
/// # 注記
/// 付随情報で南緯および西経である事が指示されている場合は、数値を負数で返す。
///
fn conv_degree(value: &exif::Field, reference: &exif::Field) -> Result<f64> {
    let reference = reference.display_value().to_string();

    if let exif::Value::Rational(ref fractions) = value.value {
        let deg = fractions[0].to_f64();
        let min = fractions[1].to_f64();
        let sec = fractions[2].to_f64();

        let sign = if reference == "S" || reference == "W" {
            -1.0
        } else {
            1.0
        };

        Ok((deg + (min / 60.0) + (sec / 3600.0)) * sign)

    } else {
        Err(anyhow!("invalid GPS location information"))
    }
}
