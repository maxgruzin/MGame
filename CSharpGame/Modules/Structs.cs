using System.Collections.Generic;

namespace CSharpGame.Modules
{
    public static class Structs
    {
        // Ported types from mdlStruct.bas
        public class Obj
        {
            public char ObjType { get; set; } = '\0';
            public int kx { get; set; }
            public int ky { get; set; }
            public byte Dop { get; set; }
        }

        public class Player
        {
            public string PName { get; set; } = string.Empty;
            public int Health { get; set; }
            public int kx { get; set; }
            public int ky { get; set; }
            public byte Orug { get; set; }
            public long Scores { get; set; }
            public int Kill { get; set; }
            public int ZTS { get; set; }
            public byte nr { get; set; }
            public byte nc { get; set; }
            public byte Org1 { get; set; }
            public byte Org2 { get; set; }
            public byte Org3 { get; set; }
            public byte Org4 { get; set; }
            public byte Org5 { get; set; }
            public int rs { get; set; }
            public byte ActiveOrug { get; set; }
            public bool IsDead { get; set; }
            public int DeathTimerMs { get; set; }
        }

        public class Pulia
        {
            public byte PType { get; set; }
            public int kx1 { get; set; }
            public int ky1 { get; set; }
            public int kx2 { get; set; }
            public int ky2 { get; set; }
            public byte Napravl { get; set; }
            public int Owner { get; set; } = -1;
            public int Vx { get; set; }
            public int Vy { get; set; }
            public int LifetimeMs { get; set; } = 3000;
        }

        // Dynamic collections to replace VB6 dynamic arrays
        public static List<Obj> Elm { get; } = new List<Obj>();
        public static List<Player> Players { get; } = new List<Player>();
        public static List<Pulia> PuliaList { get; } = new List<Pulia>();

        // Helpers that mimic VB6 ReDim Preserve / UBound usage
        public static void EnsureElmSize(int count)
        {
            while (Elm.Count < count) Elm.Add(new Obj());
        }

        public static void EnsurePlayersSize(int count)
        {
            while (Players.Count < count) Players.Add(new Player());
        }

        public static void EnsurePuliaSize(int count)
        {
            while (PuliaList.Count < count) PuliaList.Add(new Pulia());
        }

        public static int UBoundElm => Elm.Count - 1;
        public static int UBoundPlayers => Players.Count - 1;
        public static int UBoundPulia => PuliaList.Count - 1;
    }
}
