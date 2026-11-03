# exifgeo

写真の Exif に記録された GPS 情報（緯度・経度）から、撮影場所の住所を表示するコマンドラインツールです。国土地理院の逆ジオコーディング API と市町村コードデータを使用し、日本国内の住所を取得します。画像ファイルや Exif の書き換えは行いません。

## ビルド方法

Rust / Cargo と、使用する環境に対応した C/C++ リンカーなどのビルド環境を用意してください。Windows の MSVC ツールチェーンでは、Visual Studio Build Tools の C++ ビルドツールが必要です。

プロジェクトのルートディレクトリで実行します。

```console
cargo build --release
```

実行ファイルは次の場所に生成されます。

- Windows: `target\release\exifgeo.exe`
- Linux / macOS: `target/release/exifgeo`

依存クレートがローカルにない場合、ビルド時にダウンロードされます。Git が利用できる場合は、バージョン情報にコミットハッシュが埋め込まれます。取得できない場合は `unknown` になります。

## 使用方法

```console
exifgeo [OPTIONS] FILE [FILE ...]
```

GPS 情報を含む写真ファイルを1つ以上指定します。以下の例では、実行ファイルを PATH の通ったディレクトリに置いているものとします。

```console
exifgeo photo.jpg
exifgeo photo1.jpg photo2.jpg
exifgeo --log-level INFO --municd-cache ./cache/municd.json photo.jpg
```

ビルドした実行ファイルを直接使う場合、Windows の PowerShell では次のように実行できます。

```powershell
.\target\release\exifgeo.exe .\photo.jpg
```

Cargo 経由でも実行できます。`--` 以降が exifgeo に渡す引数です。

```console
cargo run --release -- --log-level INFO photo.jpg
```

出力例（住所・座標は例示）:

```text
photo.jpg
    東京都千代田区丸の内一丁目 (35.68°,139.77°)
```

標準出力にはファイル名、住所、緯度・経度が表示されます。座標の表示は小数点以下2桁ですが、API への問い合わせには Exif から読み取った座標を使用します。

### CLI オプション

| オプション | 内容 | 既定値 |
| --- | --- | --- |
| `-l, --log-level <LEVEL>` | ログレベル。`OFF`、`ERROR`、`WARN`、`INFO`、`DEBUG`、`TRACE` を指定でき、大文字・小文字は区別しません。 | `OFF` |
| `-m, --municd-cache <FILE>` | 市町村コードの JSON キャッシュファイルのパス。 | OS のキャッシュディレクトリ配下の `exifgeo/municd.json` |
| `-h, --help` | ヘルプを表示します。 | — |
| `-V, --version` | バージョンとビルド時のコミットハッシュを表示します。 | — |

ファイルは個別に指定してください。ディレクトリの再帰検索機能はありません。GPS 情報がない場合や、ファイルの読み込み・住所の問い合わせに失敗した場合は、標準エラー出力に理由を表示し、次のファイルの処理を続けます。ログレベルが `OFF` でも、これらのメッセージは表示されます。

### 設定とキャッシュファイル

専用の設定ファイルはありません。設定は CLI オプションで指定します。`municd.json` はアプリケーションが自動生成・更新する市町村コードのキャッシュで、通常は手動で編集する必要はありません。

既定の保存先は OS によって異なります。

| OS | 既定の保存先 |
| --- | --- |
| Windows | `%LOCALAPPDATA%\exifgeo\municd.json` |
| Linux | `$XDG_CACHE_HOME/exifgeo/municd.json`（未設定時は `~/.cache/exifgeo/municd.json`） |
| macOS | `~/Library/Caches/exifgeo/municd.json` |

OS のキャッシュディレクトリを取得できない場合は、カレントディレクトリの `municd.json` を使用します。実際の既定パスは `--help` で確認できます。

キャッシュには、市町村コードをキーとする JSON オブジェクトを保存します。各レコードは `code`（数値の市町村コード）、`pref`（都道府県名）、`town`（市区町村名）を持ちます。

- キャッシュがない場合、読み込めない場合、またはファイル更新から10日以上経過した場合は、市町村コードデータを再取得します。
- 保存先の親ディレクトリは必要に応じて作成されます。保存先には書き込み権限が必要です。
- キャッシュが有効でも、写真の座標から住所を取得する際には毎回 API に問い合わせるため、インターネット接続が必要です。

使用する国土地理院のエンドポイント:

- 住所の問い合わせ: `https://mreversegeocoder.gsi.go.jp/reverse-geocoder/LonLatToAddress`
- 市町村コードデータ: `https://maps.gsi.go.jp/js/muni.js`

## ライセンス

[MIT License](LICENSE)
