# ac-expander-rs

AtCoderへの提出用に、複数のファイルに分けたRustコードを1ファイルにまとめるツールです。`mod foo;` で宣言されたモジュールを再帰的に展開し、整形したコードを出力します。

## 必要なもの

- Rust / Cargo（このツールのビルドにはEdition 2024対応の環境が必要です）
- rustfmt（`cargo fmt` が実行できること）

rustupを利用している場合は、次のコマンドでrustfmtを追加できます。

```sh
rustup component add rustfmt
```

## インストール

このリポジトリのルートで実行してください。

```sh
cargo install --path . --locked
```

インストール後は `acxp` コマンドを利用できます。Cargoのインストール先（通常は `~/.cargo/bin`）を `PATH` に含めてください。

## 使い方

対象のCargoプロジェクトのルートで、入力ファイルを指定します。

```sh
# 展開結果をファイルに保存
acxp src/main.rs -o submission.rs

# 展開結果を標準出力に出力
acxp src/main.rs

# ヘルプを表示
acxp --help
```

`-o` は `--output` とも指定できます。出力先のファイルが存在する場合は上書きされます。生成された `submission.rs` の内容をAtCoderに提出してください。

インストールせず、このリポジトリから実行することもできます。

```sh
cargo run --release -- /path/to/project/src/main.rs -o /path/to/submission.rs
```

**実行時には元のソースファイルも整形されます。** 入力ファイルのディレクトリから親方向に最初に見つかる `Cargo.toml` を使って `cargo fmt` を実行するため、対象プロジェクト内の他のソースファイルも整形されることがあります。入力ファイルのディレクトリまたは祖先ディレクトリに `Cargo.toml` が必要です。

## モジュールの配置例

次のようなファイル構成に対応しています。

```text
project/
├── Cargo.toml
└── src/
    ├── main.rs       # mod math; mod utils;
    ├── math.rs
    ├── utils.rs      # pub mod input;
    └── utils/
        └── input.rs
```

`main.rs` の `mod math;` は `math.rs` の内容を含む `mod math { ... }` に置き換わります。`utils.rs` 内の `pub mod input;` も同様に展開され、モジュール構造と可視性を保った1ファイルになります。

## 制約

- 外部ファイルのモジュールは `foo.rs` 形式に対応しています。`foo/mod.rs` 形式や `#[path = "..."]` による読み込み先の指定には対応していません。
- モジュール宣言を行単位の正規表現で検出します。インラインモジュール内の外部モジュール宣言など、任意のRust構文を正確に解析するものではありません。
- `cfg` による条件付きコンパイルは評価しません。無効な条件のモジュールでも、展開対象になればファイルが必要です。
- 外部クレートや `include!` の内容は展開しません。外部クレートを使う場合は、提出先で利用できることを確認してください。
- コンパイルや動作確認は行いません。生成されたコードは提出前に別途確認してください。
