using System;
using System.Drawing;
using System.Linq;
using System.IO;
using System.Collections.Generic;

namespace CSharpGame.Modules
{
    public class GameEngine
    {
        public static GameEngine Instance { get; } = new GameEngine();

        // Minimal runtime state
        public int MaxPl { get; set; } = 0;
        public int ekrl { get; set; } = 0; // camera X flag
        public int ekrt { get; set; } = 0; // camera Y flag
        public int PI { get; set; } = 0; // current player index
        // Input flags
        public bool KeyUpFlag { get; set; }
        public bool KeyDownFlag { get; set; }
        public bool AttackFlag { get; set; }
        private DateTime _lastShot = DateTime.MinValue;

        public void Update()
        {
            // Advance projectiles (per-frame lightweight work).
            for (int i = Structs.PuliaList.Count - 1; i >= 0; i--)
            {
                var p = Structs.PuliaList[i];
                if (p.PType == 0)
                {
                    Structs.PuliaList.RemoveAt(i);
                    continue;
                }

                // update lifetime and position using velocity
                p.LifetimeMs -= 16; // approximate frame time
                p.kx1 += p.Vx;
                p.ky1 += p.Vy;

                if (p.LifetimeMs <= 0 || p.kx1 < -1000 || p.kx1 > 2000 || p.ky1 < -1000 || p.ky1 > 2000)
                {
                    Structs.PuliaList.RemoveAt(i);
                    continue;
                }

                // Determine collision targets based on projectile owner.
                if (p.Owner >= 0 && p.Owner < Structs.Players.Count && p.Owner < MaxPl)
                {
                    // projectile fired by player -> check enemies
                    for (int e = MaxPl; e < Structs.Players.Count; e++)
                    {
                        var pl = Structs.Players[e];
                        if (pl == null) continue;
                        if (pl.kx < 0 || pl.ky < 0) continue;
                        if (InRECT(pl.kx, pl.ky, 50, 50, p.kx1, p.ky1) ||
                            InRECT(pl.kx, pl.ky, 50, 50, p.kx1 + 3, p.ky1) ||
                            InRECT(pl.kx, pl.ky, 50, 50, p.kx1, p.ky1 + 3))
                        {
                            p.PType = 0;
                            pl.Health -= 100;
                            if (pl.Health <= 0)
                            {
                                // enemy died
                                int owner = p.Owner;
                                pl.PName = string.Empty;
                                pl.kx = -1;
                                pl.ky = -1;
                                pl.Health = 0;
                                EnemyCounts = Math.Max(0, EnemyCounts - 1);
                                if (owner >= 0 && owner < Structs.Players.Count)
                                {
                                    Structs.Players[owner].Scores += 10;
                                    Structs.Players[owner].Kill += 1;
                                }
                            }
                            break;
                        }
                    }
                }
                else
                {
                    // projectile fired by enemy (or unknown) -> check player slots
                    for (int e = 0; e < Math.Min(MaxPl, Structs.Players.Count); e++)
                    {
                        var pl = Structs.Players[e];
                        if (pl == null) continue;
                        if (pl.kx < 0 || pl.ky < 0) continue;
                        if (pl.IsDead) continue;
                        if (InRECT(pl.kx, pl.ky, 50, 50, p.kx1, p.ky1) ||
                            InRECT(pl.kx, pl.ky, 50, 50, p.kx1 + 3, p.ky1) ||
                            InRECT(pl.kx, pl.ky, 50, 50, p.kx1, p.ky1 + 3))
                        {
                            p.PType = 0;
                            pl.Health -= 100;
                            if (pl.Health <= 0)
                            {
                                // player hit dead: mark dead and start respawn timer
                                pl.IsDead = true;
                                pl.DeathTimerMs = 2000;
                                pl.PName = string.Empty;
                                pl.Scores = pl.Scores - 10;
                                pl.kx = -1;
                                pl.ky = -1;
                                pl.Health = 0;
                            }
                            break;
                        }
                    }
                }
            }
        }

        // Timer handlers (ported from frmStart timers)
        // EnemyCounts tracks currently active enemies; start at 0 so spawns can occur
        public int EnemyCounts { get; set; } = 0;
        private bool GameWon = false;
        private bool HasEnemiesSpawned = false;  // Track if any enemies have been spawned
        private bool GameOverShown = false;
        // track how many times each spawn-marker (Elm index) has spawned enemies
        private Dictionary<int, int> SpawnCounts { get; } = new Dictionary<int, int>();

        public void HandleShootTick()
        {
            // handle shooting cooldown and animation; movement is handled by tmrButtons to match VB6 timing
            if (PI >= 0 && PI < Structs.Players.Count)
            {
                try
                {
                    var player = Structs.Players[PI];
                    if (AttackFlag)
                    {
                        if ((DateTime.Now - _lastShot).TotalMilliseconds > 500)
                        {
                            player.nr = 9;
                            Shoot(PI, 1);
                            _lastShot = DateTime.Now;
                        }
                        else
                        {
                            player.nr = 8;
                        }
                    }
                }
                catch { }
            }
        }

        public void HandleButtonsTick()
        {
            // Ported from VB6 tmrButtons_Timer: update animation and move based on input flags
            if (PI < 0 || PI >= Structs.Players.Count) return;
            var pl = Structs.Players[PI];

            if (pl.IsDead) return;

            if (KeyUpFlag)
            {
                if (pl.nr >= 7) pl.nr = 1; else pl.nr = (byte)(pl.nr + 1);
                switch (pl.nc)
                {
                    case 1: Move(PI, -8, 0); break;
                    case 2: Move(PI, -3, 3); break;
                    case 3: Move(PI, 0, 8); break;
                    case 4: Move(PI, 3, 3); break;
                    case 5: Move(PI, 8, 0); break;
                    case 6: Move(PI, 3, -3); break;
                    case 7: Move(PI, 0, -8); break;
                    case 8: Move(PI, -3, -3); break;
                }
            }

            if (KeyDownFlag)
            {
                if (pl.nr == 1) pl.nr = 7; else pl.nr = (byte)(pl.nr - 1);
                switch (pl.nc)
                {
                    case 1: Move(PI, 4, 0); break;
                    case 2: Move(PI, 2, -2); break;
                    case 3: Move(PI, 0, -4); break;
                    case 4: Move(PI, -2, -2); break;
                    case 5: Move(PI, -2, 0); break;
                    case 6: Move(PI, -2, 2); break;
                    case 7: Move(PI, 0, 4); break;
                    case 8: Move(PI, 2, 2); break;
                }
            }
        }

        public void HandleEnemiesGoTick()
        {
            // Enemy AI step: use NextStep/MakeStep logic so enemies attack when close
            for (int i = MaxPl; i < Structs.Players.Count; i++)
            {
                if (i >= Structs.Players.Count) break;
                var enemy = Structs.Players[i];
                if (enemy == null) continue;
                if (enemy.kx < 0 || enemy.ky < 0) continue;

                // pick a nearby player (if any) and execute AI step
                int target = PoiskBlizIgroka(i);
                if (target < 0) target = 0;
                NextStep(i, target);
            }
        }

        public void HandleEnemiesTick()
        {
            // Ported spawn logic from VB6 tmrEnemys_Timer (simplified)
            // Keep global cap higher (originally 15); per-spawn-point limit remains 4
            if (EnemyCounts >= 15) return;
            for (int idx = 0; idx < Structs.Elm.Count; idx++)
            {
                var e = Structs.Elm[idx];
                if (e == null) continue;
                if (e.ObjType != '2') continue; // spawn marker

                // check adjacent tile (right)
                if (idx + 1 < Structs.Elm.Count)
                {
                    var right = Structs.Elm[idx + 1];
                    if (right.ObjType == '0' && IsPlayer(right.kx, right.ky, 50, 50))
                    {
                        // per-spawn-point limit: allow at most 4 respawns for this spawn marker
                        if (SpawnCounts.TryGetValue(idx, out var scount) && scount >= 4) continue;
                        // find free slot
                        int free = -1;
                        for (int p = MaxPl; p < Structs.Players.Count; p++) if (string.IsNullOrEmpty(Structs.Players[p].PName)) { free = p; break; }
                        if (free == -1)
                        {
                            free = Structs.Players.Count;
                            Structs.EnsurePlayersSize(free + 1);
                        }
                        int ti = (new System.Random()).Next(1, 3);
                        Structs.Players[free].PName = ti.ToString();
                        Structs.Players[free].Health = 200;
                        Structs.Players[free].kx = right.kx;
                        Structs.Players[free].ky = right.ky;
                        Structs.Players[free].nc = 1;
                        Structs.Players[free].nr = 1;
                        EnemyCounts++;
                        HasEnemiesSpawned = true;
                        // increment per-spawn counter
                        SpawnCounts[idx] = SpawnCounts.TryGetValue(idx, out var sc) ? sc + 1 : 1;
                        if (EnemyCounts >= 15) return;
                    }
                }

                // check adjacent tile (left)
                if (idx - 1 >= 0)
                {
                    var left = Structs.Elm[idx - 1];
                    if (left.ObjType == '0' && IsPlayer(left.kx, left.ky, 50, 50))
                    {
                        // per-spawn-point limit: allow at most 4 respawns for this spawn marker
                        if (SpawnCounts.TryGetValue(idx, out var scount2) && scount2 >= 4) continue;
                        int free = Structs.Players.FindIndex(p => string.IsNullOrEmpty(p.PName) && Structs.Players.IndexOf(p) >= MaxPl);
                        if (free == -1)
                        {
                            free = Structs.Players.Count;
                            Structs.EnsurePlayersSize(free + 1);
                        }
                        int ti = (new System.Random()).Next(1, 3);
                        Structs.Players[free].PName = ti.ToString();
                        Structs.Players[free].Health = 200;
                        Structs.Players[free].kx = left.kx;
                        Structs.Players[free].ky = left.ky;
                        Structs.Players[free].nc = 1;
                        Structs.Players[free].nr = 1;
                        EnemyCounts++;
                        HasEnemiesSpawned = true;
                        // increment per-spawn counter
                        SpawnCounts[idx] = SpawnCounts.TryGetValue(idx, out var sc2) ? sc2 + 1 : 1;
                        if (EnemyCounts >= 15) return;
                    }
                }
            }
        }

        public void HandleTimer1Tick()
        {
            // general periodic tasks: handle respawn timers and win detection
            for (int i = 0; i < Structs.Players.Count; i++)
            {
                var pl = Structs.Players[i];
                if (pl == null) continue;
                if (pl.IsDead)
                {
                    pl.DeathTimerMs -= 50; // timer1 interval is 50ms
                    if (pl.DeathTimerMs <= 0)
                    {
                        // Instead of respawning, show game over and return to StartForm if local player died
                        if (i == 0)
                        {
                            if (!GameOverShown)
                            {
                                GameOverShown = true;
                                System.Windows.Forms.MessageBox.Show("You died. Game over.", "Game Over", System.Windows.Forms.MessageBoxButtons.OK, System.Windows.Forms.MessageBoxIcon.Information);
                                // Close any open MainForm instances
                                foreach (Form f in System.Windows.Forms.Application.OpenForms)
                                {
                                    if (f is MainForm) f.Close();
                                }
                                // Show a new StartForm (single-time)
                                var start = new StartForm();
                                start.Show();
                            }
                            // prevent repeated popups
                            pl.IsDead = false;
                            pl.DeathTimerMs = 0;
                            return;
                        }
                        else
                        {
                            pl.IsDead = false;
                            SelectPlace(i);
                            pl.Health = 1000;
                        }
                    }
                }
            }

            // Win detection: all enemies dead (only after enemies have spawned)
            if (!GameWon && HasEnemiesSpawned && EnemyCounts <= 0 && MaxPl > 0)
            {
                GameWon = true;
                System.Windows.Forms.MessageBox.Show("Game over — all enemies defeated!", "Victory", System.Windows.Forms.MessageBoxButtons.OK, System.Windows.Forms.MessageBoxIcon.Information);
                System.Windows.Forms.Application.Exit();
            }
        }

        public int SelectPlace(int PN)
        {
            // Ported from VB6 SelectPlace: find first free spawn tile and assign to player PN
            if (PN < 0) return -1;
            if (PN < Structs.Players.Count)
            {
                Structs.Players[PN].ZTS = -1;
                Structs.Players[PN].kx = -1;
                Structs.Players[PN].ky = -1;
            }

            for (int i = 0; i < Structs.Elm.Count; i++)
            {
                if (Structs.Elm[i].ObjType == '1')
                {
                    bool taken = false;
                    for (int j = 0; j < Structs.Players.Count; j++)
                    {
                        if (Structs.Players[j].ZTS == i) { taken = true; break; }
                    }
                    if (taken) continue;

                    // assign
                    if (PN >= Structs.Players.Count) Structs.EnsurePlayersSize(PN + 1);
                    Structs.Players[PN].kx = Structs.Elm[i].kx;
                    Structs.Players[PN].ky = Structs.Elm[i].ky;
                    Structs.Players[PN].nr = 1;
                    Structs.Players[PN].nc = 3;
                    Structs.Players[PN].ZTS = i;
                    return i;
                }
            }
            return -1;
        }

        public void Render(Graphics g)
        {
            g.Clear(Color.Black);

            // Render map elements using sprite sheet if available
            foreach (var elm in Structs.Elm)
            {
                if (elm == null) continue;
                var x = elm.kx - (ekrl * 800);
                var y = elm.ky - (ekrt * 600);

                // Special-case: treat '0' as a floor tile coming from Tiles.bmp when available
                if (elm.ObjType == '0' && Sprites_Tiles != null)
                {
                    var src0 = new Rectangle(0, 0, 50, 50);
                    DirectDraw.Blit(g, Sprites_Tiles, src0, x, y);
                    continue;
                }

                // numeric tiles come from Obj.bmp
                if (char.IsDigit(elm.ObjType) && Sprites_Obj != null)
                {
                    var src = GetSrcRectForObj(elm.ObjType);
                    DirectDraw.Blit(g, Sprites_Obj, src, x, y);
                    continue;
                }

                // ascii tiles come from Tiles.bmp
                if (elm.ObjType >= 'a' && elm.ObjType <= 'y' && Sprites_Tiles != null)
                {
                    var src = GetSrcRectForObj(elm.ObjType);
                    DirectDraw.Blit(g, Sprites_Tiles, src, x, y);
                    continue;
                }

                // fallback: simple rectangle
                using var pen = new Pen(Color.DarkGreen);
                g.DrawRectangle(pen, new Rectangle(x, y, 50, 50));
            }

            // Render players (use sprite sheets if available)
            for (int i = 0; i < Structs.Players.Count; i++)
            {
                var pl = Structs.Players[i];
                if (pl == null) continue;
                var x = pl.kx - (ekrl * 800);
                var y = pl.ky - (ekrt * 600);

                // compute source rect from animation indices (nc, nr) — VB6 used 50x50 frames
                var srcRect = new Rectangle(Math.Max(0, (pl.nc - 1) * 50), Math.Max(0, (pl.nr - 1) * 50), 50, 50);

                Bitmap? sheet = null;
                if (!string.IsNullOrEmpty(pl.PName))
                {
                    if (pl.PName == "1") sheet = Sprites_Spr1;
                    else if (pl.PName == "2") sheet = Sprites_Spr2;
                    else sheet = Sprites_Spr4; // general player sheet
                }
                else
                {
                    sheet = Sprites_Spr4;
                }

                if (sheet != null)
                {
                    // clamp animation indices to available frames in the sheet so sprites don't disappear
                    int maxCols = Math.Max(1, sheet.Width / 50);
                    int maxRows = Math.Max(1, sheet.Height / 50);
                    int useNc = Math.Min(maxCols, Math.Max(1, (int)pl.nc));
                    int useNr = Math.Min(maxRows, Math.Max(1, (int)pl.nr));
                    var safeSrc = new Rectangle((useNc - 1) * 50, (useNr - 1) * 50, 50, 50);
                    DirectDraw.Blit(g, sheet, safeSrc, x, y);
                    continue;
                }

                // fallback rectangle
                using var brush = new SolidBrush(Color.CornflowerBlue);
                g.FillRectangle(brush, new Rectangle(x, y, 35, 35));
            }

            // Render pulia
            foreach (var p in Structs.PuliaList)
            {
                using var b = new SolidBrush(Color.Yellow);
                g.FillEllipse(b, p.kx1 - (ekrl * 800), p.ky1 - (ekrt * 600), 6, 6);
            }

            // If local player is dead, overlay Game Over text
            if (PI >= 0 && PI < Structs.Players.Count && Structs.Players[PI].IsDead)
            {
                using var fbig = new Font("Segoe UI", 36, FontStyle.Bold);
                using var brush = new SolidBrush(Color.FromArgb(200, Color.Black));
                g.FillRectangle(brush, 0, 200, 800, 200);
                using var brush2 = new SolidBrush(Color.White);
                g.DrawString("GAME OVER", fbig, brush2, 240f, 250f);
            }

            // Draw HUD (player health / scores)
            if (PI >= 0 && PI < Structs.Players.Count)
            {
                var me = Structs.Players[PI];
                using var f = new Font("Segoe UI", 12);
                using var brush = new SolidBrush(Color.White);
                var text = $"Health: {me.Health}   Scores: {me.Scores}   Kills: {me.Kill}";
                g.DrawString(text, f, brush, 20f, 5f);
            }
        }

        private Rectangle GetSrcRectForObj(char objType)
        {
            // Map numeric tile types to positions in Obj.bmp (matches original VB6 offsets)
            if (char.IsDigit(objType))
            {
                switch (objType)
                {
                    case '2': return new Rectangle(200, 0, 50, 50);
                    case '3': return new Rectangle(50, 0, 50, 50);
                    case '4': return new Rectangle(100, 0, 50, 50);
                    case '5': return new Rectangle(150, 0, 50, 50);
                    case '6': return new Rectangle(250, 0, 50, 50);
                    default: return new Rectangle(0, 0, 50, 50);
                }
            }

            // lower-case letters map into Tiles.bmp in a 8x3 grid (50x50 tiles)
            if (objType >= 'a' && objType <= 'y' && Sprites_Tiles != null)
            {
                int cs = objType - 'a';
                int col = cs % 8;
                int row = cs / 8;
                return new Rectangle(col * 50, row * 50, 50, 50);
            }

            return new Rectangle(0, 0, 50, 50);
        }

        // Asset storage (nullable — check before use)
        public Bitmap? Sprites_Obj { get; private set; }
        public Bitmap? Sprites_Spr1 { get; private set; }
        public Bitmap? Sprites_Spr2 { get; private set; }
        public Bitmap? Sprites_Spr3 { get; private set; }
        public Bitmap? Sprites_Spr4 { get; private set; }
        public Bitmap? Sprites_Weapons { get; private set; }
        public Bitmap? Sprites_Tiles { get; private set; }

        public void LoadSprites(string appBasePath)
        {
            try
            {
                string? spritesPath = null;
                // search upward from provided base path for a sibling "Sprites" folder
                var cur = Path.GetFullPath(appBasePath);
                for (int depth = 0; depth < 6; depth++)
                {
                    var candidate = Path.GetFullPath(Path.Combine(cur, "..", "Sprites"));
                    if (Directory.Exists(candidate)) { spritesPath = candidate; break; }
                    cur = Path.GetFullPath(Path.Combine(cur, ".."));
                }
                if (spritesPath == null)
                {
                    var fallback = Path.GetFullPath(Path.Combine(appBasePath, "..", "..", "..", "Sprites"));
                    if (Directory.Exists(fallback)) spritesPath = fallback;
                }

                if (spritesPath == null) return;

                Sprites_Obj = FileExistsLoad(spritesPath, "Obj.bmp");
                Sprites_Spr1 = FileExistsLoad(spritesPath, "spr1.bmp");
                Sprites_Spr2 = FileExistsLoad(spritesPath, "spr2.bmp");
                Sprites_Spr3 = FileExistsLoad(spritesPath, "spr3.bmp");
                Sprites_Spr4 = FileExistsLoad(spritesPath, "spr4.bmp");
                Sprites_Weapons = FileExistsLoad(spritesPath, "Weapons.bmp");
                Sprites_Tiles = FileExistsLoad(spritesPath, "Tiles.bmp");
            }
            catch (Exception)
            {
                // ignore for now
            }
        }

        // (Debug logging removed)

        private Bitmap? FileExistsLoad(string dir, string file)
        {
            var path = Path.Combine(dir, file);
            if (File.Exists(path))
            {
                try
                {
                    var bmp = DirectDraw.LoadBitmapWithColorKey(path, null);
                    // record loaded file
                    if (LoadedSpriteFiles == null) LoadedSpriteFiles = new System.Collections.Generic.List<string>();
                    LoadedSpriteFiles.Add(path + " (" + bmp.Width + "x" + bmp.Height + ")");
                    return bmp;
                }
                catch { }
            }
            return null;
        }

        // Diagnostic list of loaded sprite files (path + size)
        public System.Collections.Generic.List<string>? LoadedSpriteFiles { get; private set; }

        public string GetLoadedSpritesReport()
        {
            if (LoadedSpriteFiles == null || LoadedSpriteFiles.Count == 0) return "No sprites loaded.";
            return string.Join("\n", LoadedSpriteFiles);
        }

        // Ported helpers from mdlGame.bas
        public static bool InRECT(int Top, int Left, int Width, int Height, int X, int Y)
        {
            return (Top < X && Top + Width > X) && (Left < Y && Left + Height > Y);
        }

        public bool Move(int PlayerIndex, int Up, int Left)
        {
            if (PlayerIndex < 0 || PlayerIndex >= Structs.Players.Count) return false;
            var player = Structs.Players[PlayerIndex];
            player.ZTS = -1;

            for (int i = 0; i < Structs.Elm.Count; i++)
            {
                var e = Structs.Elm[i];
                if (e == null) continue;
                char t = e.ObjType;
                if (t >= 'a' && t <= 'y') continue; // trees are passable for player movement
                if (t == '1')
                {
                    if (InRECT(e.kx, e.ky, 50, 50, player.kx + Left, player.ky + Up) ||
                        InRECT(e.kx, e.ky, 50, 50, player.kx + Left, player.ky + Up + 35) ||
                        InRECT(e.kx, e.ky, 50, 50, player.kx + Left + 35, player.ky + Up) ||
                        InRECT(e.kx, e.ky, 50, 50, player.kx + Left + 35, player.ky + Up + 35))
                    {
                        player.ZTS = i;
                    }
                    continue;
                }

                if (t == '0') continue;
                if (t == '3')
                {
                    if (InRECT(e.kx, e.ky, 40, 40, player.kx + Left, player.ky + Up) ||
                        InRECT(e.kx, e.ky, 40, 40, player.kx + Left, player.ky + Up + 35) ||
                        InRECT(e.kx, e.ky, 40, 40, player.kx + Left + 35, player.ky + Up) ||
                        InRECT(e.kx, e.ky, 40, 40, player.kx + Left + 35, player.ky + Up + 35))
                    {
                        // Health pickup: +300 instant and remove pickup
                        player.Health = Math.Min(1000, player.Health + 300);
                        e.ObjType = '0';
                    }
                    continue;
                }

                // proposed new rect
                int candidateX = player.kx + Left;
                int candidateY = player.ky + Up;
                bool collides = (InRECT(e.kx, e.ky, 50, 50, candidateX + 15, candidateY + 15) ||
                                InRECT(e.kx, e.ky, 50, 50, candidateX, candidateY + 35) ||
                                InRECT(e.kx, e.ky, 50, 50, candidateX + 35, candidateY + 15) ||
                                InRECT(e.kx, e.ky, 50, 50, candidateX + 35, candidateY + 35));
                if (collides)
                {
                    // attempt to slide out by small steps away from the element center (original behavior)
                    int centerX = e.kx + 25;
                    int centerY = e.ky + 25;
                    double dx = candidateX - centerX;
                    double dy = candidateY - centerY;
                    if (dx == 0 && dy == 0) { dx = 1; dy = 0; }
                    double len = Math.Sqrt(dx * dx + dy * dy);
                    double nx = dx / len;
                    double ny = dy / len;
                    int step = 2;
                    bool slid = false;
                    int tryX = candidateX;
                    int tryY = candidateY;
                    for (int s = 0; s < 30; s++)
                    {
                        tryX += (int)Math.Round(nx * step);
                        tryY += (int)Math.Round(ny * step);
                        bool anyCollision = false;
                        // check against all map elements (original 50x50 footprint)
                        for (int jj = 0; jj < Structs.Elm.Count; jj++)
                        {
                            var ee = Structs.Elm[jj];
                            if (ee == null) continue;
                            if (InRECT(ee.kx, ee.ky, 50, 50, tryX + 15, tryY + 15) ||
                                InRECT(ee.kx, ee.ky, 50, 50, tryX, tryY + 35) ||
                                InRECT(ee.kx, ee.ky, 50, 50, tryX + 35, tryY + 15) ||
                                InRECT(ee.kx, ee.ky, 50, 50, tryX + 35, tryY + 35))
                            {
                                anyCollision = true;
                                break;
                            }
                        }
                        if (!anyCollision)
                        {
                            slid = true;
                            break;
                        }
                    }
                    if (!slid)
                    {
                        return false;
                    }
                    else
                    {
                        // commit slid position
                        player.kx = tryX;
                        player.ky = tryY;
                        // after moving, unstuck nearby enemies
                        UnstuckEnemiesAt(PlayerIndex);
                        // update camera flags
                        if (Structs.Players.Count > 0)
                        {
                            if (Structs.Players[0].kx - 800 <= 0) ekrl = 0; else ekrl = 1;
                            if (Structs.Players[0].ky - 600 <= 0) ekrt = 0; else ekrt = 1;
                        }
                        return true;
                    }
                }
            }

            // proposed new position
            int newX = player.kx + Left;
            int newY = player.ky + Up;

            // Check collisions against map elements (already done above) and against other players/enemies directly.
            for (int pi = 0; pi < Structs.Players.Count; pi++)
            {
                if (pi == PlayerIndex) continue;
                var op = Structs.Players[pi];
                if (op == null) continue;
                if (op.kx < 0 || op.ky < 0) continue;

                // if new position would overlap this other player
                if (InRECT(op.kx, op.ky, 35, 35, newX, newY) ||
                    InRECT(op.kx, op.ky, 35, 35, newX + 35, newY) ||
                    InRECT(op.kx, op.ky, 35, 35, newX, newY + 35) ||
                    InRECT(op.kx, op.ky, 35, 35, newX + 35, newY + 35))
                {
                        // allow move only if it increases distance (i.e., moving away), otherwise block
                        double curDist = Math.Sqrt(Math.Pow(player.kx - op.kx, 2) + Math.Pow(player.ky - op.ky, 2));
                        double newDist = Math.Sqrt(Math.Pow(newX - op.kx, 2) + Math.Pow(newY - op.ky, 2));
                        if (newDist <= curDist)
                        {
                            return false;
                        }
                        // else allow and continue checking others
                }
            }

            // commit move
            player.kx = newX;
            player.ky = newY;

            // If any enemy now overlaps player, push them away
            UnstuckEnemiesAt(PlayerIndex);

            if (Structs.Players.Count > 0)
            {
                if (Structs.Players[0].kx - 800 <= 0) ekrl = 0; else ekrl = 1;
                if (Structs.Players[0].ky - 600 <= 0) ekrt = 0; else ekrt = 1;
            }

            return true;
        }

        public bool Move2(int PlayerIndex, int Up, int Left)
        {
            if (PlayerIndex < 0 || PlayerIndex >= Structs.Players.Count) return false;
            var player = Structs.Players[PlayerIndex];
            for (int i = 0; i < Structs.Elm.Count; i++)
            {
                var e = Structs.Elm[i];
                if (e == null) continue;
                char t = e.ObjType;
                if (t >= 'a' && t <= 'y') continue; // trees are passable
                if (t == '1' || t == '0' || t == '3') continue;
                if (InRECT(e.kx, e.ky, 50, 50, player.kx + Left + 15, player.ky + Up + 15) ||
                    InRECT(e.kx, e.ky, 50, 50, player.kx + Left, player.ky + Up + 35) ||
                    InRECT(e.kx, e.ky, 50, 50, player.kx + Left + 35, player.ky + Up + 15) ||
                    InRECT(e.kx, e.ky, 50, 50, player.kx + Left + 35, player.ky + Up + 35))
                {
                    return false;
                }
            }
            return true;
        }

        public bool IsPlayer(int kx, int ky, int Width, int Height)
        {
            for (int i = 0; i < Structs.Players.Count; i++)
            {
                var p = Structs.Players[i];
                if (p.kx < 0 || p.ky < 0) continue;
                if (InRECT(kx, ky, Width, Height, p.kx, p.ky) ||
                    InRECT(kx, ky, Width, Height, p.kx, p.ky + 35) ||
                    InRECT(kx, ky, Width, Height, p.kx + 35, p.ky) ||
                    InRECT(kx, ky, Width, Height, p.kx + 35, p.ky + 35))
                {
                    return false;
                }
            }
            return true;
        }

        private void UnstuckEnemiesAt(int PlayerIndex)
        {
            var player = Structs.Players[PlayerIndex];
            if (player == null) return;
            for (int i = MaxPl; i < Structs.Players.Count; i++)
            {
                var enemy = Structs.Players[i];
                if (enemy == null) continue;
                if (enemy.kx < 0 || enemy.ky < 0) continue;
                if (!InRECT(enemy.kx, enemy.ky, 35, 35, player.kx, player.ky)) continue;
                UnstuckEnemy(i);
            }
        }

        private void UnstuckEnemy(int EnemyIndex)
        {
            if (EnemyIndex < 0 || EnemyIndex >= Structs.Players.Count) return;
            var enemy = Structs.Players[EnemyIndex];
            if (enemy == null) return;
            if (enemy.kx < 0 || enemy.ky < 0) return;

            // Determine a center to push away from: prefer nearest overlapping player, otherwise nearest overlapping tree
            int centerX = 0, centerY = 0;
            bool foundCenter = false;

            double bestDist = double.MaxValue;
            for (int i = 0; i < Structs.Players.Count; i++)
            {
                if (i == EnemyIndex) continue;
                var p = Structs.Players[i];
                if (p == null) continue;
                if (p.kx < 0 || p.ky < 0) continue;
                if (!InRECT(enemy.kx, enemy.ky, 35, 35, p.kx, p.ky)) continue;
                double d = Math.Sqrt(Math.Pow(enemy.kx - p.kx, 2) + Math.Pow(enemy.ky - p.ky, 2));
                if (d < bestDist) { bestDist = d; centerX = p.kx; centerY = p.ky; foundCenter = true; }
            }

            if (!foundCenter)
            {
                for (int ei = 0; ei < Structs.Elm.Count; ei++)
                {
                    var em = Structs.Elm[ei];
                    if (em == null) continue;
                    if (!(em.ObjType >= 'a' && em.ObjType <= 'y')) continue; // tree
                    if (!InRECT(em.kx, em.ky, 50, 50, enemy.kx, enemy.ky)) continue;
                    double d = Math.Sqrt(Math.Pow(enemy.kx - em.kx, 2) + Math.Pow(enemy.ky - em.ky, 2));
                    if (d < bestDist) { bestDist = d; centerX = em.kx; centerY = em.ky; foundCenter = true; }
                }
            }

            if (!foundCenter) return;

            // Slide enemy away from center in small steps until it's no longer overlapping player/tree or until max attempts
            double dx = enemy.kx - centerX;
            double dy = enemy.ky - centerY;
            if (dx == 0 && dy == 0) { dx = 1; dy = 0; }
            double len = Math.Sqrt(dx * dx + dy * dy);
            double nx = dx / len;
            double ny = dy / len;
            int stepSize = 2;
            int maxSteps = 40;

            for (int s = 0; s < maxSteps; s++)
            {
                enemy.kx += (int)Math.Round(nx * stepSize);
                enemy.ky += (int)Math.Round(ny * stepSize);

                bool overlap = false;
                // check players
                for (int i = 0; i < Structs.Players.Count; i++)
                {
                    if (i == EnemyIndex) continue;
                    var p = Structs.Players[i];
                    if (p == null) continue;
                    if (p.kx < 0 || p.ky < 0) continue;
                    if (InRECT(enemy.kx, enemy.ky, 35, 35, p.kx, p.ky)) { overlap = true; break; }
                }
                if (overlap) continue;

                // check trees (smaller footprint to avoid excessive sticking)
                bool onTree = false;
                for (int ei = 0; ei < Structs.Elm.Count; ei++)
                {
                    var em = Structs.Elm[ei];
                    if (em == null) continue;
                    if (!(em.ObjType >= 'a' && em.ObjType <= 'y')) continue;
                    if (InRECT(em.kx, em.ky, 50, 50, enemy.kx, enemy.ky)) { onTree = true; break; }
                }
                if (onTree) continue;

                // cleared overlaps
                return;
            }

            // fallback: teleport to nearest free if sliding didn't resolve
            TeleportEnemyToNearestFree(EnemyIndex);
        }

        private void TeleportEnemyToNearestFree(int EnemyIndex)
        {
            if (EnemyIndex < 0 || EnemyIndex >= Structs.Players.Count) return;
            var enemy = Structs.Players[EnemyIndex];
            if (enemy == null) return;

            int bestIdx = -1;
            double bestDist = double.MaxValue;

            for (int i = 0; i < Structs.Elm.Count; i++)
            {
                var e = Structs.Elm[i];
                if (e == null) continue;
                // consider floor tiles '0' and spawn points '1' as candidate free places
                if (e.ObjType != '0' && e.ObjType != '1') continue;
                // ensure no player occupies this tile
                bool taken = false;
                for (int p = 0; p < Structs.Players.Count; p++)
                {
                    var pl = Structs.Players[p];
                    if (pl == null) continue;
                    if (pl.kx == e.kx && pl.ky == e.ky) { taken = true; break; }
                }
                if (taken) continue;
                double d = Math.Sqrt(Math.Pow(enemy.kx - e.kx, 2) + Math.Pow(enemy.ky - e.ky, 2));
                if (d < bestDist) { bestDist = d; bestIdx = i; }
            }

            if (bestIdx != -1)
            {
                var dest = Structs.Elm[bestIdx];
                enemy.kx = dest.kx;
                enemy.ky = dest.ky;
            }
            else
            {
                // fallback: push enemy a bit
                enemy.kx += 40;
                enemy.ky += 40;
            }
        }
        

        public bool IgrokBlizko(int PlayerIndex, int EnemyIndex)
        {
            for (int i = MaxPl; i < Structs.Players.Count; i++)
            {
                if (InRECT(Structs.Players[PlayerIndex].kx, Structs.Players[PlayerIndex].ky, 35, 35, Structs.Players[i].kx, Structs.Players[i].ky) ||
                    InRECT(Structs.Players[PlayerIndex].kx, Structs.Players[PlayerIndex].ky, 35, 35, Structs.Players[i].kx, Structs.Players[i].ky + 35) ||
                    InRECT(Structs.Players[PlayerIndex].kx, Structs.Players[PlayerIndex].ky, 35, 35, Structs.Players[i].kx + 35, Structs.Players[i].ky) ||
                    InRECT(Structs.Players[PlayerIndex].kx, Structs.Players[PlayerIndex].ky, 35, 35, Structs.Players[i].kx + 35, Structs.Players[i].ky + 35))
                {
                    return true;
                }
            }
            return false;
        }

        public int PoiskBlizIgroka(int EnemyIndex)
        {
            int mini = -1;
            int min = int.MaxValue;
            if (EnemyIndex < 0 || EnemyIndex >= Structs.Players.Count) return -1;
            var enemy = Structs.Players[EnemyIndex];
            if (enemy.kx > -1 && enemy.ky > -1)
            {
                // Search among player slots (0 .. MaxPl-1)
                for (int i = 0; i < MaxPl; i++)
                {
                    if (i >= Structs.Players.Count) break;
                    var p = Structs.Players[i];
                    if (p.kx > -1 && p.ky > -1)
                    {
                        p.rs = (int)Math.Sqrt(Math.Pow((double)(Structs.Players[EnemyIndex].kx - p.kx), 2) + Math.Pow((double)(Structs.Players[EnemyIndex].ky - p.ky), 2));
                        if (p.rs < min) { min = p.rs; mini = i; }
                    }
                }
            }
            return mini;
        }

        public void NextStep(int EnemyIndex, int PlayerIndex)
        {
            if (EnemyIndex < 0 || EnemyIndex >= Structs.Players.Count) return;
            if (PlayerIndex < 0 || PlayerIndex >= Structs.Players.Count) return;

            var enemy = Structs.Players[EnemyIndex];
            var player = Structs.Players[PlayerIndex];

            // If enemy is overlapping the player, push the enemy away first so it doesn't get stuck
            if (enemy != null && player != null && InRECT(enemy.kx, enemy.ky, 35, 35, player.kx, player.ky))
            {
                UnstuckEnemy(EnemyIndex);
            }

            if (IgrokBlizko(PlayerIndex, EnemyIndex))
            {
                if (enemy.nr < 7) enemy.nr = 7;
                else if (enemy.nr >= 9) enemy.nr = 7;
                else enemy.nr = (byte)(enemy.nr + 1);
                player.Health = player.Health - 1;
                if (player.Health <= 0)
                {
                    // player dies immediately and is marked dead; respawn handled by timer
                    player.IsDead = true;
                    player.DeathTimerMs = 2000;
                    player.Scores = player.Scores - 10;
                    player.kx = -1;
                    player.ky = -1;
                    player.Health = 0;
                }
                return;
            }

            // Choose direction based on relative position (mirrors VB6 branching)
            if (player.kx < enemy.kx && player.ky < enemy.ky)
            {
                MakeStep(8, PlayerIndex, EnemyIndex);
                enemy.nr = (byte)(enemy.nr == 6 ? 1 : enemy.nr + 1);
            }
            else if (player.kx < enemy.kx && player.ky == enemy.ky)
            {
                MakeStep(7, PlayerIndex, EnemyIndex);
                enemy.nr = (byte)(enemy.nr == 6 ? 1 : enemy.nr + 1);
            }
            else if (player.kx == enemy.kx && player.ky < enemy.ky)
            {
                enemy.nr = (byte)(enemy.nr >= 6 ? 1 : enemy.nr + 1);
                MakeStep(1, PlayerIndex, EnemyIndex);
            }
            else if (player.kx > enemy.kx && player.ky > enemy.ky)
            {
                enemy.nr = (byte)(enemy.nr >= 6 ? 1 : enemy.nr + 1);
                MakeStep(4, PlayerIndex, EnemyIndex);
            }
            else if (player.kx == enemy.kx && player.ky > enemy.ky)
            {
                enemy.nr = (byte)(enemy.nr >= 6 ? 1 : enemy.nr + 1);
                MakeStep(5, PlayerIndex, EnemyIndex);
            }
            else if (player.kx > enemy.kx && player.ky == enemy.ky)
            {
                enemy.nr = (byte)(enemy.nr >= 6 ? 1 : enemy.nr + 1);
                MakeStep(3, PlayerIndex, EnemyIndex);
            }
            else if (player.kx > enemy.kx && player.ky < enemy.ky)
            {
                enemy.nr = (byte)(enemy.nr >= 6 ? 1 : enemy.nr + 1);
                MakeStep(2, PlayerIndex, EnemyIndex);
            }
            else if (player.kx < enemy.kx && player.ky > enemy.ky)
            {
                enemy.nr = (byte)(enemy.nr >= 6 ? 1 : enemy.nr + 1);
                MakeStep(6, PlayerIndex, EnemyIndex);
            }
        }

        private void MakeStep(int Pov, int PlayerIndex, int EnemyIndex)
        {
            if (EnemyIndex <= 0 || EnemyIndex >= Structs.Players.Count) return;
            int attempts = 0;
            int pov = Pov;
            while (attempts < 10)
            {
                attempts++;
                bool moved = false;
                switch (pov)
                {
                    case 8:
                        if (Move2(EnemyIndex, -5, -5))
                        {
                            Structs.Players[EnemyIndex].nc = 8;
                            Structs.Players[EnemyIndex].kx -= 5;
                            Structs.Players[EnemyIndex].ky -= 5;
                            moved = true;
                            UnstuckEnemy(EnemyIndex);
                        }
                        else pov = 1;
                        break;
                    case 1:
                        if (Move2(EnemyIndex, -10, 0))
                        {
                            Structs.Players[EnemyIndex].nc = 1;
                            Structs.Players[EnemyIndex].ky -= 10;
                            moved = true;
                            UnstuckEnemy(EnemyIndex);
                        }
                        else pov = 2;
                        break;
                    case 2:
                        if (Move2(EnemyIndex, -5, 5))
                        {
                            Structs.Players[EnemyIndex].nc = 2;
                            Structs.Players[EnemyIndex].kx += 6;
                            Structs.Players[EnemyIndex].ky -= 6;
                            moved = true;
                            UnstuckEnemy(EnemyIndex);
                        }
                        else pov = 3;
                        break;
                    case 7:
                        if (Move2(EnemyIndex, -10, 0))
                        {
                            Structs.Players[EnemyIndex].nc = 7;
                            Structs.Players[EnemyIndex].kx -= 10;
                            moved = true;
                            UnstuckEnemy(EnemyIndex);
                        }
                        else pov = 8;
                        break;
                    case 4:
                        if (Move2(EnemyIndex, 5, 5))
                        {
                            Structs.Players[EnemyIndex].nc = 4;
                            Structs.Players[EnemyIndex].kx += 6;
                            Structs.Players[EnemyIndex].ky += 6;
                            moved = true;
                            UnstuckEnemy(EnemyIndex);
                        }
                        else pov = 5;
                        break;
                    case 5:
                        if (Move2(EnemyIndex, 10, 0))
                        {
                            Structs.Players[EnemyIndex].nc = 5;
                            Structs.Players[EnemyIndex].ky += 10;
                            moved = true;
                            UnstuckEnemy(EnemyIndex);
                        }
                        else pov = 6;
                        break;
                    case 3:
                        if (Move2(EnemyIndex, 0, 10))
                        {
                            Structs.Players[EnemyIndex].nc = 3;
                            Structs.Players[EnemyIndex].kx += 10;
                            moved = true;
                            UnstuckEnemy(EnemyIndex);
                        }
                        else pov = 4;
                        break;
                    case 6:
                        if (Move2(EnemyIndex, 5, -5))
                        {
                            Structs.Players[EnemyIndex].nc = 6;
                            Structs.Players[EnemyIndex].kx -= 5;
                            Structs.Players[EnemyIndex].ky += 5;
                            moved = true;
                            UnstuckEnemy(EnemyIndex);
                        }
                        else pov = 7;
                        break;
                    default:
                        pov = (pov % 8) + 1;
                        break;
                }

                if (moved)
                {
                    // try to unstuck the enemy from any overlapping players or map obstacles
                    UnstuckEnemy(EnemyIndex);
                    bool overlapsPlayer = false;
                    for (int pi = 0; pi < Structs.Players.Count; pi++)
                    {
                        if (pi == EnemyIndex) continue;
                        var p = Structs.Players[pi];
                        if (p == null) continue;
                        if (p.kx < 0 || p.ky < 0) continue;
                        if (InRECT(Structs.Players[EnemyIndex].kx, Structs.Players[EnemyIndex].ky, 35, 35, p.kx, p.ky)) { overlapsPlayer = true; break; }
                    }
                    if (overlapsPlayer)
                    {
                        // try to push enemy further away; if still overlapping, teleport to nearest free floor
                        UnstuckEnemy(EnemyIndex);
                        bool still = false;
                        for (int pi = 0; pi < Structs.Players.Count; pi++)
                        {
                            if (pi == EnemyIndex) continue;
                            var p = Structs.Players[pi];
                            if (p == null) continue;
                            if (InRECT(Structs.Players[EnemyIndex].kx, Structs.Players[EnemyIndex].ky, 35, 35, p.kx, p.ky)) { still = true; break; }
                        }
                        if (still) TeleportEnemyToNearestFree(EnemyIndex);
                    }
                    // also if enemy sits on an impassable map tile (tree/obstacle), teleport it
                    for (int ei = 0; ei < Structs.Elm.Count; ei++)
                    {
                        var em = Structs.Elm[ei];
                        if (em == null) continue;
                        if (em.ObjType >= 'a' && em.ObjType <= 'y')
                        {
                            if (InRECT(em.kx, em.ky, 50, 50, Structs.Players[EnemyIndex].kx, Structs.Players[EnemyIndex].ky)) { TeleportEnemyToNearestFree(EnemyIndex); break; }
                        }
                    }
                    break;
                }
            }
            // if after trying directions enemy still didn't move, try to unstuck it
            if (!Structs.Players[EnemyIndex].IsDead)
            {
                if (!Structs.Players[EnemyIndex].kx.Equals(null) && !Structs.Players[EnemyIndex].ky.Equals(null))
                {
                    // check if enemy overlaps impassable map elements (trees or obstacles)
                    bool onTree = false;
                    for (int ei = 0; ei < Structs.Elm.Count; ei++)
                    {
                        var em = Structs.Elm[ei];
                        if (em == null) continue;
                        if (em.ObjType >= 'a' && em.ObjType <= 'y')
                        {
                            if (InRECT(em.kx, em.ky, 50, 50, Structs.Players[EnemyIndex].kx, Structs.Players[EnemyIndex].ky)) { onTree = true; break; }
                        }
                    }
                    if (onTree) UnstuckEnemy(EnemyIndex);
                }
            }
        }

        public void Shoot(int PlayerIndex, byte Orug)
        {
            // Find free pulia or add
            var free = Structs.PuliaList.FirstOrDefault(p => p.PType == 0);
            if (free == null)
            {
                var p = new Structs.Pulia();
                Structs.PuliaList.Add(p);
                free = p;
            }
            ProvPov(Structs.PuliaList.IndexOf(free), PlayerIndex, Orug);
        }

        public void RotateLeft()
        {
            if (PI >= 0 && PI < Structs.Players.Count)
            {
                var p = Structs.Players[PI];
                if (p.nc == 1) p.nc = 8; else p.nc = (byte)(p.nc - 1);
            }
        }

        public void RotateRight()
        {
            if (PI >= 0 && PI < Structs.Players.Count)
            {
                var p = Structs.Players[PI];
                if (p.nc == 8) p.nc = 1; else p.nc = (byte)(p.nc + 1);
            }
        }

        private void ProvPov(int Index, int PlayerIndex, byte Orug)
        {
            var pov = Structs.Players[PlayerIndex].nc;
            var pul = Structs.PuliaList[Index];
            pul.Owner = PlayerIndex;
            pul.Napravl = (byte)pov;
            pul.LifetimeMs = 3000; // 3 seconds
            if (Orug == 1)
            {
                switch (pov)
                {
                    case 1:
                        pul.PType = 1;
                        pul.kx1 = Structs.Players[PlayerIndex].kx + 15;
                        pul.kx2 = pul.kx1;
                        pul.ky1 = Structs.Players[PlayerIndex].ky;
                        pul.ky2 = pul.ky1 - 20;
                        pul.Vx = 0;
                        pul.Vy = -10;
                        break;
                    case 2:
                        pul.PType = 1;
                        pul.kx1 = Structs.Players[PlayerIndex].kx + 27;
                        pul.kx2 = pul.kx1 + 27;
                        pul.ky1 = Structs.Players[PlayerIndex].ky + 12;
                        pul.ky2 = pul.ky1 - 15;
                        pul.Vx = 6;
                        pul.Vy = -6;
                        break;
                    case 3:
                        pul.PType = 1;
                        pul.kx1 = Structs.Players[PlayerIndex].kx + 50;
                        pul.kx2 = Structs.Players[PlayerIndex].kx + 20 + 50;
                        pul.ky1 = Structs.Players[PlayerIndex].ky + 20;
                        pul.ky2 = pul.ky1;
                        pul.Vx = 10;
                        pul.Vy = 0;
                        break;
                    case 4:
                        pul.PType = 1;
                        pul.kx1 = Structs.Players[PlayerIndex].kx + 50;
                        pul.kx2 = Structs.Players[PlayerIndex].kx + 15 + 50;
                        pul.ky1 = Structs.Players[PlayerIndex].ky + 40;
                        pul.ky2 = pul.ky1 + 15;
                        pul.Vx = 6;
                        pul.Vy = 6;
                        break;
                    case 5:
                        pul.PType = 1;
                        pul.kx1 = Structs.Players[PlayerIndex].kx + 25;
                        pul.kx2 = pul.kx1;
                        pul.ky1 = Structs.Players[PlayerIndex].ky + 40;
                        pul.ky2 = pul.ky1 + 15;
                        pul.Vx = 0;
                        pul.Vy = 10;
                        break;
                    case 6:
                        pul.PType = 1;
                        pul.kx1 = Structs.Players[PlayerIndex].kx - 10;
                        pul.kx2 = Structs.Players[PlayerIndex].kx - 20;
                        pul.ky1 = Structs.Players[PlayerIndex].ky + 50;
                        pul.ky2 = pul.ky1 + 15;
                        pul.Vx = -6;
                        pul.Vy = 6;
                        break;
                    case 7:
                        pul.PType = 1;
                        pul.kx1 = Structs.Players[PlayerIndex].kx;
                        pul.kx2 = Structs.Players[PlayerIndex].kx - 10;
                        pul.ky1 = Structs.Players[PlayerIndex].ky + 30;
                        pul.ky2 = pul.ky1;
                        pul.Vx = -10;
                        pul.Vy = 0;
                        break;
                    case 8:
                        pul.PType = 1;
                        pul.kx1 = Structs.Players[PlayerIndex].kx - 10;
                        pul.kx2 = Structs.Players[PlayerIndex].kx - 20;
                        pul.ky1 = Structs.Players[PlayerIndex].ky - 10;
                        pul.ky2 = pul.ky1 - 15;
                        pul.Vx = -6;
                        pul.Vy = -6;
                        break;
                }
            }
        }
    }
}
