# 1. ビルダーステージ
FROM rust:latest as builder

# ビルドに必要な依存関係をインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# アプリケーションのソースコードをコピー
COPY . .

# ビルド
# backend ディレクトリに移動してビルドする
RUN cd backend && cargo fix --allow-dirty --allow-staged && cargo build --release

# 2. 実行ステージ
FROM debian:bookworm-slim as runner

# 実行に必要なライブラリをインストール
RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl-dev \
    libsqlite3-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ビルドされたバイナリをコピー
COPY --from=builder /app/backend/target/release/md-wiki-backend .

# migrationsとstorageディレクトリを作成
RUN mkdir -p /app/migrations /app/storage

# 実行ユーザーを作成し、権限を付与
RUN useradd -m -s /bin/bash appuser && \
    chown -R appuser:appuser /app

USER appuser

# ポートを公開
EXPOSE 3000

# 起動コマンド
CMD ["./md-wiki-backend"]
