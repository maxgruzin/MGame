using System;
using System.Drawing;
using System.IO;

namespace CSharpGame.Modules
{
    public static class DirectDraw
    {
        // Load a bitmap and apply a color key transparency (e.g., white)
        public static Bitmap LoadBitmapWithColorKey(string path, Color? colorKey = null)
        {
            if (!File.Exists(path)) throw new FileNotFoundException(path);
            var bmp = new Bitmap(path);
            try
            {
                // If no explicit color key provided, use the top-left pixel as transparent key
                if (!colorKey.HasValue)
                {
                    var key = bmp.GetPixel(0, 0);
                    bmp.MakeTransparent(key);
                }
                else
                {
                    bmp.MakeTransparent(colorKey.Value);
                }
            }
            catch
            {
                // ignore transparency failures
            }
            return bmp;
        }

        // Blit a portion of source bitmap onto destination graphics at (x,y)
        public static void Blit(Graphics g, Bitmap src, Rectangle srcRect, int x, int y)
        {
            if (src == null || g == null) return;
            g.DrawImage(src, new Rectangle(x, y, srcRect.Width, srcRect.Height), srcRect, GraphicsUnit.Pixel);
        }
    }
}
