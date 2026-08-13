# 签名与供应链

`Config\greendev\supply-chain-policy.json` 保存代码签名、分离签名、可信/吊销证书指纹、可选系统证书链校验、密钥轮换周期和许可证规则。

发布时传入 `release.ps1 -SignThumbprint THUMBPRINT` 会对 GUI、CLI、安装器做 Authenticode 签名，并为发布制品、`release-manifest.json` 和更新 Feed 生成 RSA-PSS-SHA256 `.sig.json`。签名工具也可用于 Profile 和离线介质：

```powershell
Scripts\sign-greendev-artifact.ps1 -Path FILE -Thumbprint THUMBPRINT
Scripts\verify-greendev-signature.ps1 -Path FILE -PolicyPath Config\greendev\supply-chain-policy.json
```

每个发布目录同时包含完整 npm/Cargo/环境组件的 CycloneDX 1.5 `release-sbom.cdx.json` 与带输入材料摘要的 `provenance.json`。验证器会交叉检查签名算法、文件名、SHA256、证书指纹、签署时间、信任/吊销列表；`requireTrustedChain` 启用后还会验证 Windows 系统证书链。没有证书时保持本地未签名状态；启用强制策略后，验证门禁会要求相应签名。
