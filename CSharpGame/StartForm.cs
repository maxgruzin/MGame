using System;
using System.IO;
using System.Linq;
using System.Windows.Forms;
using CSharpGame.Modules;

namespace CSharpGame
{
    public class StartForm : Form
    {
        private ListBox lstMaps;
        private TextBox txtName;
        private Button btnCreate;
        private Button btnExit;
        private TextBox txtHost;
        private Button btnJoin;

        // Timers from VB6
        private System.Windows.Forms.Timer tmrShoot;
        private System.Windows.Forms.Timer tmrButtons;
        private System.Windows.Forms.Timer tmrEnemysGo;
        private System.Windows.Forms.Timer tmrEnemys;
        private System.Windows.Forms.Timer timer1;

        private NetworkManager _net = new NetworkManager();
        private bool SndOK = false;
        private string _currentMapPath;

        public StartForm()
        {
            Text = "Start - C# Port";
            ClientSize = new System.Drawing.Size(800, 600);

            lstMaps = new ListBox() { Left = 20, Top = 20, Width = 360, Height = 400 };
            txtName = new TextBox() { Left = 400, Top = 20, Width = 360, Text = "Unknown" };
            btnCreate = new Button() { Left = 400, Top = 60, Width = 160, Text = "Create" };
            btnExit = new Button() { Left = 600, Top = 60, Width = 160, Text = "Exit" };
            txtHost = new TextBox() { Left = 20, Top = 440, Width = 260, Text = "localhost" };
            btnJoin = new Button() { Left = 300, Top = 440, Width = 120, Text = "Join" };

            Controls.Add(lstMaps);
            Controls.Add(txtName);
            Controls.Add(btnCreate);
            Controls.Add(btnExit);
            Controls.Add(txtHost);
            Controls.Add(btnJoin);

            Load += StartForm_Load;
            btnCreate.Click += BtnCreate_Click;
            btnExit.Click += (s, e) => Application.Exit();
            btnJoin.Click += BtnJoin_Click;

            // setup timers
            tmrShoot = new System.Windows.Forms.Timer() { Interval = 20 };
            tmrShoot.Tick += TmrShoot_Tick;
            tmrButtons = new System.Windows.Forms.Timer() { Interval = 35 };
            tmrButtons.Tick += TmrButtons_Tick;
            tmrEnemysGo = new System.Windows.Forms.Timer() { Interval = 80 };
            tmrEnemysGo.Tick += TmrEnemysGo_Tick;
            tmrEnemys = new System.Windows.Forms.Timer() { Interval = 2000 };
            tmrEnemys.Tick += TmrEnemys_Tick;
            timer1 = new System.Windows.Forms.Timer() { Interval = 50 };
            timer1.Tick += Timer1_Tick;

            // network events
            _net.ClientConnected += Net_ClientConnected;
            _net.DataReceived += Net_DataReceived;
            _net.ErrorOccurred += Net_ErrorOccurred;
        }

        private string? FindMapsDirectory()
        {
            // Search upward from the executable directory for a sibling "Maps" folder
            var cur = AppDomain.CurrentDomain.BaseDirectory;
            for (int depth = 0; depth < 6; depth++)
            {
                var candidate = Path.GetFullPath(Path.Combine(cur, "..", "Maps"));
                if (Directory.Exists(candidate)) return candidate;
                cur = Path.GetFullPath(Path.Combine(cur, ".."));
            }
            // final fallback: look three levels up (typical when running from bin folder)
            var fallback = Path.GetFullPath(Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "..", "..", "..", "Maps"));
            if (Directory.Exists(fallback)) return fallback;
            return null;
        }

        private void StartForm_Load(object sender, EventArgs e)
        {
            var mapsPath = FindMapsDirectory();
            if (string.IsNullOrEmpty(mapsPath))
            {
                MessageBox.Show("Maps folder not found. Expected a 'Maps' directory in the project root.", "Maps Missing", MessageBoxButtons.OK, MessageBoxIcon.Warning);
                return;
            }
            var files = Directory.GetFiles(mapsPath, "*.mmp");
            foreach (var f in files)
            {
                lstMaps.Items.Add(Path.GetFileName(f));
            }
        }

        private void BtnCreate_Click(object sender, EventArgs e)
        {
            if (lstMaps.SelectedItem == null)
            {
                MessageBox.Show("Select a map first.");
                return;
            }
            var selected = lstMaps.SelectedItem.ToString();
            var mapsPath = FindMapsDirectory();
            if (string.IsNullOrEmpty(mapsPath)) { MessageBox.Show("Maps folder not found."); return; }
            var full = Path.Combine(mapsPath, selected);
            LoadMap(full);
            _currentMapPath = full;

            // Count players and initialize
            int count = Structs.Elm.Count(x => x.ObjType == '1');
            GameEngine.Instance.MaxPl = count;
            // Reserve players: players + 15 enemies
            Structs.EnsurePlayersSize(count + 15);

            // reset enemy counter so spawning can begin
            GameEngine.Instance.EnemyCounts = 0;

            // Initialize player spawn positions from map
            int piIndex = 0;
            for (int i = 0; i < Structs.Elm.Count && piIndex < count; i++)
            {
                var elm = Structs.Elm[i];
                if (elm.ObjType == '1')
                {
                    var pl = Structs.Players[piIndex];
                    pl.kx = elm.kx;
                    pl.ky = elm.ky;
                    pl.PName = (piIndex == 0) ? txtName.Text : $"Enemy{piIndex}";
                    pl.ZTS = -1;
                    pl.Health = 1000;
                    pl.ActiveOrug = 1;
                    piIndex++;
                }
            }

            // mark remaining players (enemies) as inactive positions
            for (int i = count; i < Structs.Players.Count; i++)
            {
                Structs.Players[i].kx = -1;
                Structs.Players[i].ky = -1;
            }

            // Setup camera flags
            if (Structs.Players.Count > 0)
            {
                var p0 = Structs.Players[0];
                GameEngine.Instance.ekrl = (p0.kx - 800 <= 0) ? 0 : 1;
                GameEngine.Instance.ekrt = (p0.ky - 600 <= 0) ? 0 : 1;
                GameEngine.Instance.PI = 0;
            }

            // Start server listening on default port (1596)
            _net.StartServer(1596);

            // Start timers similar to VB6 behavior
            tmrButtons.Start();
            tmrEnemysGo.Start();
            tmrShoot.Start();
            tmrEnemys.Start();
            timer1.Start();

            // Load sprites (no startup popup)
            GameEngine.Instance.LoadSprites(AppDomain.CurrentDomain.BaseDirectory);

            // Open main form
            var main = new MainForm();
            main.Show();
            Hide();
        }

        private async void BtnJoin_Click(object sender, EventArgs e)
        {
            var host = txtHost.Text.Trim();
            if (string.IsNullOrEmpty(host)) return;
            await _net.ConnectToServer(host, 1596);
        }

        private void Net_ClientConnected(System.Net.Sockets.TcpClient client)
        {
            // If we're a client (not running server), send our name to the server
            if (!_net.IsServerRunning)
            {
                _net.Send(client, "Name=" + txtName.Text);
            }
            // If we are server, wait for client to send its Name= message
        }

        private void Net_DataReceived(System.Net.Sockets.TcpClient client, string data)
        {
            if (string.IsNullOrWhiteSpace(data)) return;
            data = data.Trim();

            // Client received OK from server
            if (!_net.IsServerRunning && data.Equals("ok", StringComparison.OrdinalIgnoreCase))
            {
                SndOK = true;
                return;
            }

            // Client received a map payload
            if (!_net.IsServerRunning && data.StartsWith("MAP|", StringComparison.OrdinalIgnoreCase))
            {
                try
                {
                    var b64 = data.Substring(4);
                    var bytes = Convert.FromBase64String(b64);
                    var mapsDir = System.IO.Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "..", "Maps");
                    System.IO.Directory.CreateDirectory(mapsDir);
                    var outPath = System.IO.Path.Combine(mapsDir, "received_map.mmp");
                    System.IO.File.WriteAllBytes(outPath, bytes);
                    // Load map on UI thread
                    if (InvokeRequired)
                    {
                        BeginInvoke(new Action(() => LoadMap(outPath)));
                    }
                    else
                    {
                        LoadMap(outPath);
                    }
                }
                catch { }
                return;
            }

            // Server received a Name= message from a client
            if (_net.IsServerRunning && data.StartsWith("Name=", StringComparison.OrdinalIgnoreCase))
            {
                var name = data.Substring(5);
                // register player in next available player slot (after host spawn slots)
                int countSpawns = GameEngine.Instance.MaxPl;
                Structs.EnsurePlayersSize(countSpawns + 15);
                int slot = -1;
                for (int i = countSpawns; i < Structs.Players.Count; i++)
                {
                    if (Structs.Players[i].kx <= -1)
                    {
                        slot = i; break;
                    }
                }
                if (slot == -1)
                {
                    // no free slot, try append
                    slot = Structs.Players.Count;
                    Structs.EnsurePlayersSize(slot + 1);
                }
                var pl = Structs.Players[slot];
                pl.PName = name;
                pl.ZTS = -1;
                pl.kx = -1;
                pl.ky = -1;
                pl.Health = 1000;
                // send acknowledgment back to client
                _net.Send(client, "ok");

                // send map data to client if available
                try
                {
                    if (!string.IsNullOrEmpty(_currentMapPath) && System.IO.File.Exists(_currentMapPath))
                    {
                        var bytes = System.IO.File.ReadAllBytes(_currentMapPath);
                        var b64 = Convert.ToBase64String(bytes);
                        // send as MAP|base64
                        _net.Send(client, "MAP|" + b64);
                    }
                }
                catch { }
            }
        }

        private void Net_ErrorOccurred(Exception ex)
        {
            // minimal error reporting
        }

        private void LoadMap(string address)
        {
            try
            {
                Structs.Elm.Clear();
                using var sr = new StreamReader(address);
                for (int yy = 0; yy < 24; yy++)
                {
                    var line = sr.ReadLine();
                    if (line == null) break;
                    for (int xx = 0; xx < Math.Min(32, line.Length); xx++)
                    {
                        var z = line[xx];
                        var o = new Structs.Obj() { ObjType = z, kx = xx * 50, ky = yy * 50 };
                        Structs.Elm.Add(o);
                    }
                }
            }
            catch (Exception ex)
            {
                MessageBox.Show("Failed to load map: " + ex.Message);
            }
        }

        private void TmrShoot_Tick(object sender, EventArgs e)
        {
            // delegate to GameEngine for high-frequency tasks
            GameEngine.Instance.HandleShootTick();
        }

        private void TmrButtons_Tick(object sender, EventArgs e)
        {
            GameEngine.Instance.HandleButtonsTick();
        }

        private void TmrEnemysGo_Tick(object sender, EventArgs e)
        {
            GameEngine.Instance.HandleEnemiesGoTick();
        }

        private void TmrEnemys_Tick(object sender, EventArgs e)
        {
            GameEngine.Instance.HandleEnemiesTick();
        }

        private void Timer1_Tick(object sender, EventArgs e)
        {
            GameEngine.Instance.HandleTimer1Tick();
        }
    }
}
