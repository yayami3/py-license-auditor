use serde::{Deserialize, Serialize};

/// ライセンスルール: 完全一致とパターンマッチングをサポート
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LicenseRule {
    /// 完全一致するライセンス名のリスト
    #[serde(default)]
    pub exact: Vec<String>,
    /// Globパターン（例: "GPL-*", "BSD-*"）
    #[serde(default)]
    pub patterns: Vec<String>,
}

/// パッケージ固有の例外設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageException {
    pub name: String,
    pub version: Option<String>,
    pub reason: String,
}

/// ライセンスポリシー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensePolicy {
    /// ポリシー名
    pub name: String,
    /// ポリシーの説明
    pub description: Option<String>,
    /// 許可されたライセンス
    #[serde(default)]
    pub allowed_licenses: LicenseRule,
    /// 禁止されたライセンス
    #[serde(default)]
    pub forbidden_licenses: LicenseRule,
    /// レビューが必要なライセンス
    #[serde(default)]
    pub review_required: LicenseRule,
    /// パッケージ固有の例外
    #[serde(default)]
    pub exceptions: Vec<PackageException>,
}
