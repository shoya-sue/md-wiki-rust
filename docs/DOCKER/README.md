# MD Wiki Rust - Docker環境構築ガイド

このプロジェクトはWSL AlmaLinux 9上でDockerを使用して構築できます。バックエンドはRust（Axum）、フロントエンドはReact（Tauri）で構成されています。

## 前提条件

- WSL2がインストールされていること
- AlmaLinux 9がWSLにインストールされていること
- Docker Engine と Docker Composeがインストールされていること

## WSLにAlmaLinux 9をインストールする

まだAlmaLinux 9がWSLにインストールされていない場合は、以下の手順でインストールします：

1. PowerShellを管理者権限で開き、以下を実行します：
```powershell
wsl --install -d AlmaLinux
```

もしAlmaLinuxがリストにない場合は、[AlmaLinux WSL](https://github.com/almalinux/wsl-images)から手動でインストールできます。

## Docker環境の構築

### 1. AlmaLinux 9にDockerをインストールする

WSLのAlmaLinux 9シェルで以下のコマンドを実行します：

```bash
# リポジトリを追加
sudo dnf config-manager --add-repo=https://download.docker.com/linux/centos/docker-ce.repo

# Dockerパッケージをインストール
sudo dnf install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Dockerサービスを開始および自動起動に設定
sudo systemctl enable --now docker

# 現在のユーザーをdockerグループに追加
sudo usermod -aG docker $USER

# 変更を適用するために再ログイン
newgrp docker
```

### 2. プロジェクトをビルドして実行する

```bash
# プロジェクトディレクトリに移動
cd /path/to/md-wiki-rust

# バックエンドサービスをビルドして起動
docker-compose up -d backend

# ログを確認
docker-compose logs -f backend
```

## フロントエンド開発（Tauri）

Tauriはデスクトップアプリケーションを構築するためのフレームワークで、通常はDockerコンテナ内では実行されません。ローカル環境で開発する必要があります：

### WindowsのネイティブNode.js環境でのTauri開発

1. [Node.js](https://nodejs.org/)をインストール
2. [Rust](https://www.rust-lang.org/tools/install)をインストール
3. [Tauriの依存関係](https://tauri.app/v1/guides/getting-started/prerequisites)をインストール
4. フロントエンドディレクトリに移動して依存関係をインストール：

```bash
cd frontend
npm install
npm run tauri dev
```

## 注意事項

- JWT_SECRETは本番環境では必ず安全な値に変更してください
- データベースは`storage`ディレクトリにマウントされます
- バックエンドAPIはポート3000で公開されます

## トラブルシューティング

### Dockerコンテナが起動しない場合

```bash
# ログを確認
docker-compose logs backend

# コンテナ内でシェルを実行
docker-compose exec backend /bin/bash
```

### WSLでのDockerデーモン起動エラー

WSL2でDockerデーモンが起動しない場合は、以下を試してください：

```bash
# WSL内でsystemdを有効にする
# /etc/wsl.confに以下を追加（WSL2の場合）
[boot]
systemd=true

# WSLを再起動
wsl --shutdown
# その後WSLを再度開く
```
