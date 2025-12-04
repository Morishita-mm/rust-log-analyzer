# --- ビルドステージ ---
FROM python:3.11-slim AS builder

# uvのインストール
COPY --from=ghcr.io/astral-sh/uv:latest /uv /bin/uv

WORKDIR /app

# 依存関係ファイルのコピー
COPY pyproject.toml uv.lock ./

# 仮想環境を作成し、依存パッケージをインストール
# --frozen: uv.lockの内容を厳密に守る
RUN uv sync --frozen

# --- 実行ステージ ---
FROM python:3.11-slim

WORKDIR /app

# ビルドステージから仮想環境を丸ごとコピー
COPY --from=builder /app/.venv /app/.venv

# アプリケーションコードをコピー
COPY app.py .

# 標準出力のバッファリングを無効化
ENV PYTHONUNBUFFERED=1

# 仮想環境のPythonを使って実行するようにPATHを設定
ENV PATH="/app/.venv/bin:$PATH"

# Flaskアプリの実行
CMD ["python", "app.py"]