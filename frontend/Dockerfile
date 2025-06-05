FROM almalinux:9 as builder

# Node.js と npm のインストール
RUN dnf -y update && dnf -y install \
    curl \
    gcc-c++ \
    make \
    git \
    && dnf clean all

# Node.jsのインストール
RUN curl -fsSL https://rpm.nodesource.com/setup_20.x | bash - && \
    dnf -y install nodejs

# 開発用のTauri依存関係をインストール
RUN dnf -y groupinstall "Development Tools" && \
    dnf -y install \
    webkit2gtk3-devel \
    gtk3-devel \
    libsoup-devel \
    openssl-devel \
    tauri-devel \
    && dnf clean all

# アプリケーションディレクトリの作成
WORKDIR /app

# package.jsonをコピー
COPY package.json package-lock.json* ./

# 依存関係のインストール
RUN npm ci

# ソースコードをコピー
COPY . .

# ビルド（開発環境用）
# RUN npm run build

# 開発サーバーの設定
EXPOSE 1420

# 開発サーバーを起動
CMD ["npm", "run", "dev"]
