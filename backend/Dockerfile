FROM almalinux:9 as builder

# 開発ツールのインストール
RUN dnf -y update && dnf -y install --allowerasing \
    gcc \
    make \
    pkg-config \
    openssl-devel \
    sqlite-devel \
    git \
    curl \
    && dnf clean all

# Rustのインストール
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# アプリケーションディレクトリの作成
WORKDIR /app

# Cargo.toml と Cargo.lock をコピー (依存関係のキャッシュのため)
COPY Cargo.toml Cargo.lock ./

# 実際のソースコード全体をコピー
COPY . .

# ビルド
RUN cargo build --release && \
    ls -la target/release/

# 実行環境
FROM almalinux:9

# 必要なランタイム依存関係をインストール
RUN dnf -y update && dnf -y install --allowerasing \
    openssl \
    sqlite \
    git \
    ca-certificates \
    && dnf clean all

# アプリケーションディレクトリの作成
WORKDIR /app

# バイナリをコピー
COPY --from=builder /app/target/release/md-wiki-rust-backend /app/

# migrationsディレクトリを作成（コンテナ内で必要な場合）
RUN mkdir -p /app/migrations

# ストレージディレクトリを作成
RUN mkdir -p /app/storage

# 実行ユーザーの作成
RUN useradd -m appuser
RUN chown -R appuser:appuser /app
USER appuser

# ポートを公開
EXPOSE 3000

# デバッグ用：実行ファイルが存在するか確認
RUN ls -la /app

# 起動コマンド
CMD ["sh", "-c", "cd /app && ./md-wiki-rust-backend"]
