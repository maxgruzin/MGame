using System;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace CSharpGame
{
    static class Program
    {
        [STAThread]
        static void Main(string[] args)
        {
            if (args != null && args.Length > 0 && args[0].Equals("test-network", StringComparison.OrdinalIgnoreCase))
            {
                // run local network test
                TestNetworkRunner.RunTestAsync().GetAwaiter().GetResult();
                return;
            }

            ApplicationConfiguration.Initialize();
            Application.Run(new StartForm());
        }
    }
}
