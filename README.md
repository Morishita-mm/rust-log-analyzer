# RustLogStream (仮)

**リアルタイム・ログ分析CUIツール**
Rust (TUI/非同期), Python (データ分析), Redis (メッセージブローカー) を用いたマイクロサービスアーキテクチャの実証プロジェクト。

---

![Demo](docs/images/demo.gif)

## 概要 (Overview)

複数のコンテナやサービスから発生する大量のログをリアルタイムに収集・表示し、その場で分析・調査を行うためのターミナルベース(CUI)のツールです。

**主な特徴:**
* **リアルタイム性:** Redis Pub/Subにより、ログの発生から表示までを低遅延で実現。
* **非同期I/O:** Rust (Tokio) による非ブロッキングな実装で、大量のログが流れてもUIがフリーズしません。
* **強力な分析:** Python (Polars) をバックエンドに用い、高速なリアルタイム集計（ウィンドウ集計など）を提供。
* **インタラクティブな調査機能:** 正規表現によるリアルタイムフィルタリング、Vim風キー操作によるスクロール、OSC 52対応のクリップボードコピー機能を搭載。

## アーキテクチャ (Architecture)

本プロジェクトは、関心の分離とスケーラビリティを意識したメッセージ駆動型マイクロサービスアーキテクチャを採用しています。

```mermaid
graph TD
    %% --- サブグラフ定義 ---

    %% 1. ログソース（プロデューサー）
    subgraph "Log Sources (Producers)"
        style Dummy fill:#f9f,stroke:#333,stroke-width:2px
        Dummy["dummy_log_sender.py (Python Script)"]
    end

    %% 2. メッセージブローカー（ミドルウェア）
    subgraph "Message Broker"
        style Redis fill:#ff9,stroke:#f66,stroke-width:4px,stroke-dasharray: 5 5
        Redis[(Redis Pub/Sub)]
    end

    %% 3. マイクロサービス（コンシューマー＆プロセッサー）
    subgraph "Microservices"
        style Analyzer fill:#ccf,stroke:#333,stroke-width:2px
        style Collector fill:#cfc,stroke:#333,stroke-width:2px
        
        Analyzer["Python Analyzer (Polars / Backend)"]
        Collector["Rust Collector (Tokio / Ratatui TUI / Frontend)"]
    end

    %% 4. ユーザーインターフェース
    Terminal["User Terminal (CUI Display)"]

    %% --- データフローの定義 ---

    %% Flow 1: Raw Logs Injection
    Dummy -->|Publish Raw JSON Logs| Redis

    %% Flow 2: Raw Logs Consumption
    Redis -- "Channel: logs.ingest" --> Analyzer
    Redis -- "Channel: logs.ingest" --> Collector

    %% Flow 3: Real-time aggregation processing
    %% Analyzer -- Processing: Window Aggregation (1s) --> Analyzer

    %% Flow 4: Aggregated Stats Publishing & Consumption
    Analyzer -->|"Publish Aggregated Stats JSON"| Redis
    Redis -- "Channel: stats.update" --> Collector

    %% Flow 5: User Interface Rendering
    Collector -->|"Render TUI & Handle Input"| Terminal
````

  * **Rust Collector:** TUIの描画、ユーザー入力の処理、Redisからのデータ受信を担当。非同期ランタイムTokio上で動作し、RatatuiでUIを構築。
  * **Python Analyzer:** 生ログを受信し、Polarsライブラリを用いてリアルタイムに統計情報（エラーレート、トップサービスなど）を集計し、Redisに書き戻す。
  * **Redis:** 各コンポーネント間を疎結合につなぐメッセージブローカーとして機能。

## 使用技術 (Tech Stack)

  * **Rust (Collector):**
      * `tokio`: 非同期ランタイム
      * `ratatui` / `crossterm`: TUI構築
      * `redis-rs`: Redisクライアント
      * `serde` / `serde_json`: JSONシリアライズ/デシリアライズ
      * `regex`: 正規表現エンジン
  * **Python (Analyzer):**
      * `polars`: 高速データフレームライブラリ
      * `redis-py`: Redisクライアント
  * **Infrastructure / Tools:**
      * Redis
      * Docker / Docker Compose
      * uv (Pythonパッケージ管理)

## セットアップと実行方法 (Getting Started)

**前提条件:**

  * Docker および Docker Compose がインストールされていること。
  * VS Code と Dev Containers 拡張機能の使用を推奨します。

**手順:**

1.  リポジトリをクローンします。
    ```bash
    git clone [https://github.com/Morishita-mm/rust-log-analyzer.git](https://github.com/Morishita-mm/rust-log-analyzer.git)
    ```
2.  VS Codeでフォルダを開き、推奨されるDev Container環境で再起動します（環境構築が自動で行われます）。
3.  ターミナルを3つ開き、以下の順序で実行します。
      * **Terminal 1 (Redis):** (Dev Container使用時は自動起動しているため不要。手動の場合は `docker compose up -d redis`)
      * **Terminal 2 (Log Sender):** ダミーログを生成します。
        ```bash
        cd analyzer && uv run log_sender.py
        ```
      * **Terminal 3 (Analyzer):** ログを集計します。
        ```bash
        cd analyzer && uv run main.py
        ```
      * **Terminal 4 (Collector):** TUIを起動します。
        ```bash
        cd collector && cargo run
        ```

## 使い方 (Usage / Key Bindings)

| キー | モード | 説明 |
| :--- | :--- | :--- |
| `i` | 閲覧(Normal) | 入力モード(Editing)へ切り替え |
| `j` / `↓` | 閲覧(Normal) | ログリストを下にスクロール |
| `k` / `↑` | 閲覧(Normal) | ログリストを上にスクロール |
| `c` | 閲覧(Normal) | 選択中のログをクリップボードにコピー |
| `q` | 閲覧(Normal) | アプリケーションを終了 |
| `Enter` | 入力(Editing) | フィルタを確定して適用し、閲覧モードへ戻る |
| `Esc` | 入力(Editing) | フィルタ編集をキャンセルし、閲覧モードへ戻る |

## 今後の展望 (Future Roadmap)

  * [ ] 実際のDocker/Kubernetes環境からのログ収集に対応（Fluentd/Fluent Bit連携など）
  * [ ] ログのカラーリング設定の外部ファイル化
  * [ ] 統計情報のグラフ表示（ヒストグラムなど）
  * [ ] エラーハンドリングと再接続処理の強化

<!-- end list -->
