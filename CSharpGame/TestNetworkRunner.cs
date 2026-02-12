using System;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CSharpGame.Modules;

public static class TestNetworkRunner
{
    public static async Task RunTestAsync()
    {
        Console.WriteLine("Starting local map-transfer test...");
        var mapsDir = Path.Combine(AppDomain.CurrentDomain.BaseDirectory, "..", "Maps");
        if (!Directory.Exists(mapsDir))
        {
            Console.WriteLine("Maps directory not found: " + mapsDir);
            return;
        }
        var map = Directory.GetFiles(mapsDir, "*.mmp").FirstOrDefault();
        if (map == null)
        {
            Console.WriteLine("No .mmp map files found in: " + mapsDir);
            return;
        }
        Console.WriteLine("Using map: " + map);

        var server = new NetworkManager();
        var client = new NetworkManager();

        var tcs = new TaskCompletionSource<bool>();

        server.ClientConnected += (tcp) =>
        {
            Console.WriteLine("Server: client connected");
            // send map as MAP|base64 framed message
            try
            {
                var bytes = File.ReadAllBytes(map);
                var b64 = Convert.ToBase64String(bytes);
                server.Send(tcp, "MAP|" + b64);
                Console.WriteLine("Server: sent MAP payload (base64), length=" + b64.Length);
            }
            catch (Exception ex)
            {
                Console.WriteLine("Server send error: " + ex.Message);
            }
        };

        client.DataReceived += (tcp, data) =>
        {
            Console.WriteLine("Client received message of length " + data.Length);
            if (data.StartsWith("MAP|"))
            {
                try
                {
                    var b64 = data.Substring(4);
                    var bytes = Convert.FromBase64String(b64);
                    var outPath = Path.Combine(mapsDir, "received_map_test.mmp");
                    File.WriteAllBytes(outPath, bytes);
                    Console.WriteLine("Client: map written to " + outPath);
                    tcs.TrySetResult(true);
                }
                catch (Exception ex)
                {
                    Console.WriteLine("Client decode error: " + ex.Message);
                    tcs.TrySetResult(false);
                }
            }
        };

        server.ErrorOccurred += (ex) => Console.WriteLine("Server error: " + ex.Message);
        client.ErrorOccurred += (ex) => Console.WriteLine("Client error: " + ex.Message);

        server.StartServer(1596);
        Console.WriteLine("Server listening on port 1596");

        // give server a moment
        await Task.Delay(200);

        await client.ConnectToServer("127.0.0.1", 1596);
        Console.WriteLine("Client attempted connect to server");

        // wait up to 10s for map
        var completed = await Task.WhenAny(tcs.Task, Task.Delay(10000));
        if (completed == tcs.Task && tcs.Task.Result)
        {
            Console.WriteLine("Map transfer test succeeded.");
        }
        else
        {
            Console.WriteLine("Map transfer test failed or timed out.");
        }

        // cleanup
        server.Dispose();
        client.Dispose();
    }
}
