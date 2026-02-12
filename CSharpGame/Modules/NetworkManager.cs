using System;
using System.Collections.Concurrent;
using System.Collections.Generic;
using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Threading.Tasks;

namespace CSharpGame.Modules
{
    public class NetworkManager : IDisposable
    {
        private TcpListener _listener;
        private readonly List<TcpClient> _clients = new List<TcpClient>();
        private readonly ConcurrentQueue<(TcpClient, string)> _recvQueue = new ConcurrentQueue<(TcpClient, string)>();

        public event Action<TcpClient> ClientConnected;
        public event Action<TcpClient, string> DataReceived;
        public event Action<Exception> ErrorOccurred;

        public bool IsServerRunning => _listener != null;

        public void StartServer(int port)
        {
            try
            {
                _listener = new TcpListener(IPAddress.Any, port);
                _listener.Start();
                Task.Run(AcceptLoop);
            }
            catch (Exception ex)
            {
                ErrorOccurred?.Invoke(ex);
            }
        }

        private async Task AcceptLoop()
        {
            try
            {
                while (_listener != null)
                {
                    var client = await _listener.AcceptTcpClientAsync().ConfigureAwait(false);
                    lock (_clients) { _clients.Add(client); }
                    ClientConnected?.Invoke(client);
                    _ = Task.Run(() => ClientReceiveLoop(client));
                }
            }
            catch (Exception ex)
            {
                ErrorOccurred?.Invoke(ex);
            }
        }

        private async Task ClientReceiveLoop(TcpClient client)
        {
            var stream = client.GetStream();
            var readBuffer = new byte[8192];
            var buffer = new List<byte>();
            try
            {
                while (client.Connected)
                {
                    int read = await stream.ReadAsync(readBuffer.AsMemory(0, readBuffer.Length)).ConfigureAwait(false);
                    if (read == 0) break; // disconnected
                    buffer.AddRange(readBuffer.AsSpan(0, read).ToArray());

                    // parse framed messages: [4-byte length][payload]
                    while (buffer.Count >= 4)
                    {
                        var lenBytes = buffer.GetRange(0, 4).ToArray();
                        int msgLen = BitConverter.ToInt32(lenBytes, 0); // little-endian
                        if (msgLen <= 0 || msgLen > 10_000_000) // sanity cap ~10MB
                        {
                            // invalid, drop connection
                            throw new InvalidOperationException("Invalid message length");
                        }
                        if (buffer.Count >= 4 + msgLen)
                        {
                            var payload = buffer.GetRange(4, msgLen).ToArray();
                            var text = Encoding.UTF8.GetString(payload);
                            DataReceived?.Invoke(client, text);
                            buffer.RemoveRange(0, 4 + msgLen);
                        }
                        else
                        {
                            break; // wait for more data
                        }
                    }
                }
            }
            catch (Exception ex)
            {
                ErrorOccurred?.Invoke(ex);
            }
            finally
            {
                lock (_clients) { _clients.Remove(client); }
                try { stream.Close(); } catch { }
            }
        }

        public async Task ConnectToServer(string host, int port)
        {
            try
            {
                var client = new TcpClient();
                await client.ConnectAsync(host, port).ConfigureAwait(false);
                lock (_clients) { _clients.Add(client); }
                ClientConnected?.Invoke(client);
                _ = Task.Run(() => ClientReceiveLoop(client));
            }
            catch (Exception ex)
            {
                ErrorOccurred?.Invoke(ex);
            }
        }

        private async Task SendFramedAsync(TcpClient client, byte[] payload)
        {
            try
            {
                if (client?.Connected != true) return;
                var stream = client.GetStream();
                var len = BitConverter.GetBytes(payload.Length); // little-endian
                await stream.WriteAsync(len, 0, len.Length).ConfigureAwait(false);
                await stream.WriteAsync(payload, 0, payload.Length).ConfigureAwait(false);
            }
            catch (Exception ex)
            {
                ErrorOccurred?.Invoke(ex);
            }
        }

        public void SendToAll(string message)
        {
            var data = Encoding.UTF8.GetBytes(message);
            lock (_clients)
            {
                foreach (var c in _clients.ToArray())
                {
                    _ = SendFramedAsync(c, data);
                }
            }
        }

        public void Send(TcpClient client, string message)
        {
            try
            {
                if (client?.Connected == true)
                {
                    var data = Encoding.UTF8.GetBytes(message);
                    _ = SendFramedAsync(client, data);
                }
            }
            catch (Exception ex)
            {
                ErrorOccurred?.Invoke(ex);
            }
        }

        public void Dispose()
        {
            try
            {
                _listener?.Stop();
            }
            catch { }
            lock (_clients)
            {
                foreach (var c in _clients) try { c.Close(); } catch { }
                _clients.Clear();
            }
        }
    }
}
