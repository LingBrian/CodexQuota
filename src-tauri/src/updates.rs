use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManifest {
    pub version: String,
    pub target: String,
    pub url: String,
    pub sha256: String,
    pub signature: String,
}
fn version(value: &str) -> Option<[u32; 3]> {
    let parts = value
        .strip_prefix('v')
        .unwrap_or(value)
        .split('.')
        .map(|part| part.parse::<u32>().ok())
        .collect::<Option<Vec<_>>>()?;
    parts.try_into().ok()
}
impl UpdateManifest {
    pub fn validate(&self, current: &str) -> Result<(), String> {
        if version(&self.version).ok_or("更新版本号无效")?
            <= version(current).ok_or("当前版本号无效")?
        {
            return Err("没有更高版本".into());
        }
        if self.target != "windows-x86_64"
            || !self.url.starts_with("https://github.com/")
            || self.url.contains(['\r', '\n', ' '])
            || self.sha256.len() != 64
            || !self.sha256.bytes().all(|b| b.is_ascii_hexdigit())
            || self.signature.trim().is_empty()
        {
            return Err("更新元数据缺少目标、HTTPS 下载地址、校验值或签名".into());
        }
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn refuses_unsigned_or_downgrade_metadata() {
        let mut manifest = UpdateManifest {
            version: "0.0.30".into(),
            target: "windows-x86_64".into(),
            url: "https://github.com/example/CodexQuota/releases/download/v0.0.30/setup.exe".into(),
            sha256: "a".repeat(64),
            signature: String::new(),
        };
        assert!(manifest.validate("0.0.29").is_err());
        manifest.signature = "format-only-test".into();
        assert!(manifest.validate("0.0.30").is_err());
        assert!(manifest.validate("0.0.29").is_ok());
    }
}
