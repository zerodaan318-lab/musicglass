"""MusicGlass UI 程序化校准：参考图 vs 当前窗口截图。
参考图：Gemini_Generated_Image_e9m217e9m217e9m2.jpg (1244x849)
截图：传入路径（应为 1244x849，否则自动 resize 到 1244x849 比较趋势）
输出各区域平均色差，定位布局/亮度偏差。
"""
import sys
from PIL import Image
import numpy as np

REF = r"D:\OneDrive\桌面\music class\Gemini_Generated_Image_e9m217e9m217e9m2.jpg"

def reg(arr, x0, y0, x1, y1):
    return arr[y0:y1, x0:x1].reshape(-1, 3)

def main():
    shot_path = sys.argv[1] if len(sys.argv) > 1 else None
    if not shot_path:
        print("usage: compare_ui.py <screenshot.png>")
        return
    ref = np.asarray(Image.open(REF).convert("RGB")).astype(float)
    shot_raw = np.asarray(Image.open(shot_path).convert("RGB"))
    # 统一到参考高度（参考 848，截图可能 849）
    sh, sw = shot_raw.shape[0], shot_raw.shape[1]
    shot_img = Image.fromarray(shot_raw).resize((1244, 848))
    shot = np.asarray(shot_img).astype(float)
    rh, rw = ref.shape[0], ref.shape[1]

    regions = {
        "sidebar": (0, 0, 269, rh),
        "main": (269, 0, rw, rh),
        "header": (300, 0, 944, 140),
        "upload_card": (305, 170, 305 + 902, 170 + 381),
        "formats_card": (305, 579, 305 + 902, 579 + 152),
        "bg_bottomright": (900, 600, rw, rh),
        "top_left_glow": (0, 0, 200, 200),
        "bottom_left": (0, rh - 200, 180, rh),
    }
    print(f"{'region':16} {'ref_mean':18} {'shot_mean':18} {'diff':6}")
    for name, (x0, y0, x1, y1) in regions.items():
        x1 = min(x1, rw); y1 = min(y1, rh)
        r = reg(ref, x0, y0, x1, y1).mean(0)
        s = reg(shot, x0, y0, x1, y1).mean(0)
        d = np.abs(r - s).mean()
        print(f"{name:16} {str(r.round(1)):18} {str(s.round(1)):18} {d:6.1f}")

    full = np.abs(ref - shot).mean()
    print(f"\nFULL mean abs diff: {full:.1f}")
    print("ref mean ", ref.reshape(-1, 3).mean(0).round(1))
    print("shot mean", shot.reshape(-1, 3).mean(0).round(1))

if __name__ == "__main__":
    main()
