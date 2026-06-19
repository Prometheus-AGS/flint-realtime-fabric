// Smoke-test subscriber for the C# SDK.
// Usage: FRF_GATEWAY_URL=http://localhost:4000 dotnet run --project tests/e2e/csharp/
//
// Exits 0 when it receives at least one EventEnvelope within the timeout.
using System;
using System.Threading;
using System.Threading.Tasks;
using PrometheusAgs.Frf.Flint.V1.Client;

var gatewayUrl = Environment.GetEnvironmentVariable("FRF_GATEWAY_URL") ?? "http://localhost:4000";
var channelId = Environment.GetEnvironmentVariable("FRF_CHANNEL_ID") ?? "00000000-0000-0000-0000-000000000001";

using var cts = new CancellationTokenSource(TimeSpan.FromSeconds(15));
await using var client = SpineClient.Create(gatewayUrl);

var request = new PrometheusAgs.Frf.Flint.V1.SubscribeRequest
{
    ChannelId = channelId,
    ConsumerId = "e2e-csharp-smoke",
};

Console.WriteLine("Subscribed; waiting for event…");

await foreach (var envelope in client.SubscribeAsync(request, cts.Token))
{
    Console.WriteLine($"OK — received envelope id={envelope.Id} kind={envelope.Kind}");
    return;
}

Console.Error.WriteLine("Stream ended without receiving any event");
Environment.Exit(1);
