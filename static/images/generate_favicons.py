#!/usr/bin/env python3
"""
Generate favicons and PWA icons from logo.png
Requires: pip install Pillow
"""

from PIL import Image
import os

# Source image
SOURCE = "logo.png"
OUTPUT_DIR = "static/images"

# Favicon sizes
FAVICON_SIZES = {
    "favicon-16x16.png": (16, 16),
    "favicon-32x32.png": (32, 32),
    "favicon.ico": (32, 32),  # Will be saved as PNG
}

# PWA/App icon sizes
ICON_SIZES = {
    "icon-72x72.png": (72, 72),
    "icon-96x96.png": (96, 96),
    "icon-128x128.png": (128, 128),
    "icon-144x144.png": (144, 144),
    "icon-152x152.png": (152, 152),
    "apple-touch-icon.png": (180, 180),
    "icon-192x192.png": (192, 192),
    "icon-384x384.png": (384, 384),
    "icon-512x512.png": (512, 512),
}

def generate_icons():
    if not os.path.exists(SOURCE):
        print(f"Error: {SOURCE} not found!")
        return
    
    # Open source image
    with Image.open(SOURCE) as img:
        # Convert to RGBA if necessary
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        
        # Generate favicons
        print("Generating favicons...")
        for filename, size in FAVICON_SIZES.items():
            resized = img.resize(size, Image.Resampling.LANCZOS)
            output_path = os.path.join(OUTPUT_DIR, filename)
            
            if filename.endswith('.ico'):
                # For ICO, we need to save as PNG with .ico extension
                # Most modern browsers accept PNG favicons
                resized.save(output_path.replace('.ico', '.png'))
                print(f"  Created: {filename} (as PNG)")
            else:
                resized.save(output_path)
                print(f"  Created: {filename}")
        
        # Generate PWA icons
        print("\nGenerating PWA icons...")
        for filename, size in ICON_SIZES.items():
            resized = img.resize(size, Image.Resampling.LANCZOS)
            output_path = os.path.join(OUTPUT_DIR, filename)
            resized.save(output_path)
            print(f"  Created: {filename} ({size[0]}x{size[1]})")
        
        # Generate maskable icon (with padding for safe zone)
        print("\nGenerating maskable icon...")
        maskable_size = (512, 512)
        padding = int(maskable_size[0] * 0.1)  # 10% padding
        content_size = maskable_size[0] - (padding * 2)
        
        resized = img.resize((content_size, content_size), Image.Resampling.LANCZOS)
        maskable = Image.new('RGBA', maskable_size, (0, 0, 0, 0))
        maskable.paste(resized, (padding, padding))
        maskable.save(os.path.join(OUTPUT_DIR, "icon-maskable.png"))
        print(f"  Created: icon-maskable.png (512x512 with 10% padding)")
    
    print("\n✅ All icons generated successfully!")
    print("\nFiles created:")
    for f in sorted(os.listdir(OUTPUT_DIR)):
        if f.startswith(('favicon', 'icon', 'apple-touch')):
            size = os.path.getsize(os.path.join(OUTPUT_DIR, f))
            print(f"  - {f} ({size/1024:.1f} KB)")

if __name__ == "__main__":
    try:
        from PIL import Image
    except ImportError:
        print("Error: Pillow is not installed.")
        print("Install it with: pip install Pillow")
        exit(1)
    
    generate_icons()
