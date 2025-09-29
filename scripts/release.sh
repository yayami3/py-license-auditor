#!/bin/bash
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.3.3"
    exit 1
fi

echo "🚀 Starting release process for v$VERSION..."

# 1. Git状態確認
echo "🔍 Checking git status..."
if ! git diff --quiet || ! git diff --cached --quiet; then
    echo "❌ Working directory not clean. Please commit or stash changes."
    exit 1
fi

# 2. バージョン更新
echo "📝 Updating version to $VERSION..."
sed -i "s/version = \".*\"/version = \"$VERSION\"/g" Cargo.toml pyproject.toml python/py_license_auditor/__init__.py

# 3. クリーンビルドテスト
echo "📦 Testing clean build..."
cargo clean
cargo build --release

# 4. テスト実行
echo "🧪 Running tests..."
cargo test

# 5. クリーンクローンテスト
echo "🧹 Testing clean clone build..."
TEMP_DIR=$(mktemp -d)
git stash push -m "temp stash for release"
git clone . "$TEMP_DIR" 2>/dev/null

if ! (cd "$TEMP_DIR" && cargo build --release); then
    echo "❌ Clean clone build failed!"
    rm -rf "$TEMP_DIR"
    git stash pop
    exit 1
fi

rm -rf "$TEMP_DIR"
git stash pop

# 6. 全変更をコミット
echo "💾 Committing changes..."
git add -A
git commit -m "bump: v$VERSION - automated release"

# 7. タグ作成
echo "🏷️  Creating tag..."
git tag "v$VERSION"

# 8. プッシュ
echo "🚀 Pushing to remote..."
git push origin main --tags

echo "✅ Release v$VERSION completed!"
echo "🌐 GitHub Actions will build multi-platform binaries automatically."
