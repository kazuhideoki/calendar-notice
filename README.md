# Calendar Notice

## 概要

Google Calendar の予定を取得し、通知を行うプログラムです。

## 使い方

### 1. GCP で OAUTH2 クライアント ID を取得

1. https://console.cloud.google.com/apis/credentials で作成
  - 承認済みのリダイレクト URI を `http://localhost:8990/auth`(デフォルトであれば)に設定
2. 作成したクライアント ID を `oauth_secret.json` に保存

### 2. 環境変数の準備

```
cp .env.sample .env
```

### 3. 起動

```
cargo run
```
