#!/usr/bin/env python3
"""
简单的 FLUX 图标创建器
使用 PIL 库创建基于 FLUX 配色的简单图标
"""

try:
    from PIL import Image, ImageDraw, ImageFont
    import os
    
    def create_flux_icon(size):
        # 创建透明背景图像
        img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
        draw = ImageDraw.Draw(img)
        
        # FLUX 配色
        colors = [
            (102, 126, 234),  # #667eea
            (118, 75, 162),   # #764ba2  
            (240, 147, 251),  # #f093fb
            (79, 172, 254),   # #4facfe
            (0, 242, 254),    # #00f2fe
        ]
        
        # 绘制渐变圆圈作为背景
        center = size // 2
        radius = size // 2 - 10
        
        # 绘制主圆圈
        draw.ellipse([center-radius, center-radius, center+radius, center+radius], 
                    fill=colors[0] + (200,), outline=colors[2] + (255,), width=3)
        
        # 绘制内部装饰圆
        inner_radius = radius // 2
        draw.ellipse([center-inner_radius, center-inner_radius, center+inner_radius, center+inner_radius], 
                    fill=colors[3] + (150,), outline=colors[4] + (255,), width=2)
        
        # 绘制中心点
        dot_radius = size // 20
        draw.ellipse([center-dot_radius, center-dot_radius, center+dot_radius, center+dot_radius], 
                    fill=colors[1] + (255,))
        
        # 尝试添加 FLUX 文字（如果尺寸足够大）
        if size >= 64:
            try:
                # 尝试使用系统字体
                font_size = max(8, size // 8)
                font = ImageFont.truetype("arial.ttf", font_size)
            except:
                font = ImageFont.load_default()
            
            text = "FLUX"
            # 计算文字位置
            text_bbox = draw.textbbox((0, 0), text, font=font)
            text_width = text_bbox[2] - text_bbox[0]
            text_height = text_bbox[3] - text_bbox[1]
            text_x = center - text_width // 2
            text_y = center + radius // 2
            
            # 绘制文字阴影
            draw.text((text_x + 1, text_y + 1), text, font=font, fill=(0, 0, 0, 128))
            # 绘制文字
            draw.text((text_x, text_y), text, font=font, fill=colors[2] + (255,))
        
        return img
    
    def create_all_sizes():
        sizes = [256, 128, 64, 32, 16]
        base_path = r"C:\Users\TUF\Desktop\资金追踪"
        
        for size in sizes:
            img = create_flux_icon(size)
            filename = f"flux-icon-{size}.png"
            filepath = os.path.join(base_path, filename)
            img.save(filepath, 'PNG')
            print(f"创建了 {filename}")
    
    if __name__ == "__main__":
        create_all_sizes()
        print("所有图标尺寸已创建完成!")
        print("请将 flux-icon-256.png 或其他合适尺寸转换为 ICO 格式")

except ImportError:
    print("需要安装 PIL 库：pip install Pillow")
    print("或者使用在线转换工具处理 flux-icon.svg 文件")