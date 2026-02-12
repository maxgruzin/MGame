using System;
using System.Drawing;
using System.Windows.Forms;
using CSharpGame.Modules;

namespace CSharpGame
{
    public class MainForm : Form
    {
        private System.Windows.Forms.Timer _timer;

        public MainForm()
        {
            Text = "C# Port - Game";
            ClientSize = new Size(800, 600);
            DoubleBuffered = true;

            KeyPreview = true;
            this.KeyDown += MainForm_KeyDown;
            this.KeyUp += MainForm_KeyUp;

            _timer = new System.Windows.Forms.Timer();
            _timer.Interval = 16;
            _timer.Tick += (s, e) => { Modules.GameEngine.Instance.Update(); Invalidate(); };
            _timer.Start();
            // ensure closing main form exits the application
            this.FormClosed += (s, e) => Application.Exit();
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            base.OnPaint(e);
            Modules.GameEngine.Instance.Render(e.Graphics);
        }

        private void MainForm_KeyDown(object sender, KeyEventArgs e)
        {
            var ge = Modules.GameEngine.Instance;
            if (e.KeyCode == Keys.Up) ge.KeyUpFlag = true;
            if (e.KeyCode == Keys.Down) ge.KeyDownFlag = true;
            if (e.KeyCode == Keys.Left) ge.RotateLeft();
            if (e.KeyCode == Keys.Right) ge.RotateRight();
            if (e.KeyCode == Keys.ControlKey || e.KeyCode == Keys.Control) ge.AttackFlag = true;
        }

        private void MainForm_KeyUp(object sender, KeyEventArgs e)
        {
            var ge = Modules.GameEngine.Instance;
            if (e.KeyCode == Keys.Up) ge.KeyUpFlag = false;
            if (e.KeyCode == Keys.Down) ge.KeyDownFlag = false;
            if (e.KeyCode == Keys.ControlKey || e.KeyCode == Keys.Control)
            {
                ge.AttackFlag = false;
                // reset player animation state
                if (ge.PI >= 0 && ge.PI < Structs.Players.Count) Structs.Players[ge.PI].nr = 1;
            }
        }
    }
}
