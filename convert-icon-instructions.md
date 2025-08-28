# FLUX Logo 图标转换指南

已创建 `flux-icon.svg` 文件，包含适合用作应用图标的 FLUX logo。

## 转换步骤

1. 使用在线 SVG 到 ICO 转换器（如 convertio.co 或 cloudconvert.com）
2. 上传 `flux-icon.svg` 文件
3. 转换为 ICO 格式，尺寸建议：256x256, 128x128, 64x64, 32x32, 16x16
4. 下载转换后的 ICO 文件
5. 将文件重命名为 `icon.ico`
6. 替换 `tauri-app/src-tauri/icons/icon.ico` 文件

## 备用方案

如果有 ImageMagick 或其他图像处理工具：
```bash
magick flux-icon.svg -resize 256x256 -background transparent icon.ico
```

## 配置已更新

- Tauri 配置文件中已将应用名称更改为 "FLUX分析系统"
- 图标路径指向 `icons/icon.ico`
- 重启开发服务器后将生效