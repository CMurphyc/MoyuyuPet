# Live2D Tororo & Hijiki

MoyuyuPet 的 Live2D 角色使用 Live2D Inc. 提供的官方示例模型 **Tororo & Hijiki**（白猫 Tororo）。

- 模型页面：https://www.live2d.com/en/learn/sample/tororo-hijiki/
- 免费素材授权：https://www.live2d.com/eula/live2d-free-material-license-agreement_en.html
- 示例数据使用条件：https://www.live2d.com/learn/sample/model-terms/
- Cubism Core 授权：https://www.live2d.com/eula/live2d-proprietary-software-license-agreement_en.html

## 使用方式

模型文件和 Cubism Core 不随仓库分发，需在本地执行：

```powershell
powershell -ExecutionPolicy Bypass -File scripts/fetch-live2d.ps1 -AcceptLicense
```

脚本会把运行时文件下载到 `public/live2d/`，该目录已在 `.gitignore` 中忽略。未下载或加载失败时，应用会自动回退到内置 SVG 宠物。

## 署名

- 插画、建模：Live2D Inc.
- 原始模型：Tororo & Hijiki（Cubism 3.2 runtime）
- Cubism Core：Live2D Inc.

使用或分发包含该模型的应用前，请阅读并遵守上述授权条款。一般用户和小规模事业者在同意条款后可进行商用或非商用创作；中大型事业者的限制以官方条款为准。
