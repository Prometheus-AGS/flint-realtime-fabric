using PrometheusAgs.Frf.Flint.V1;
using Grpc.Core;
using Grpc.Net.Client;
using System.Runtime.CompilerServices;

namespace PrometheusAgs.Frf.Flint.V1.Client;

public sealed class SpineClient : IDisposable
{
    private readonly GrpcChannel _channel;
    private readonly SpineService.SpineServiceClient _inner;

    private SpineClient(GrpcChannel channel)
    {
        _channel = channel;
        _inner = new SpineService.SpineServiceClient(channel);
    }

    public static SpineClient Create(string address)
    {
        var channel = GrpcChannel.ForAddress(address);
        return new SpineClient(channel);
    }

    public async Task<PublishResponse> PublishAsync(
        EventEnvelope envelope,
        CancellationToken ct = default)
    {
        var response = await _inner.PublishAsync(
            new PublishRequest { Envelope = envelope },
            cancellationToken: ct);
        return response;
    }

    public async IAsyncEnumerable<EventEnvelope> SubscribeAsync(
        SubscribeRequest request,
        [EnumeratorCancellation] CancellationToken ct = default)
    {
        using var call = _inner.Subscribe(request, cancellationToken: ct);
        while (await call.ResponseStream.MoveNext(ct))
        {
            yield return call.ResponseStream.Current;
        }
    }

    /// <summary>
    /// Reconnecting subscription. On a transport error or a cleanly-closed stream
    /// (e.g. a gateway restart) it reconnects with exponential backoff and resumes
    /// from last-seen-offset + 1, so no event is skipped or replayed. Semantics
    /// mirror the Rust, TS, and Go SDKs.
    /// </summary>
    public async IAsyncEnumerable<EventEnvelope> SubscribeResilientAsync(
        SubscribeRequest request,
        ReconnectPolicy? policy = null,
        [EnumeratorCancellation] CancellationToken ct = default)
    {
        var p = policy ?? ReconnectPolicy.Default;
        ulong fromValue = request.From?.Value ?? 0UL;
        var backoff = p.InitialBackoff;
        var retries = 0;

        while (!ct.IsCancellationRequested)
        {
            var attempt = new SubscribeRequest
            {
                ChannelId = request.ChannelId,
                ConsumerId = request.ConsumerId,
                From = new Offset { Value = fromValue },
            };

            // Open the stream in a try so a connect failure triggers backoff; the
            // yield loop itself cannot live inside try/catch in C#.
            AsyncServerStreamingCall<EventEnvelope>? call = null;
            try
            {
                call = _inner.Subscribe(attempt, cancellationToken: ct);
                backoff = p.InitialBackoff;
                retries = 0;
            }
            catch (RpcException) when (!ct.IsCancellationRequested)
            {
                retries++;
                if (p.MaxRetries is int max && retries > max)
                {
                    throw;
                }
                await Task.Delay(backoff, ct);
                backoff = NextBackoff(backoff, p);
                continue;
            }

            var streamEnded = false;
            using (call)
            {
                while (!streamEnded)
                {
                    bool moved;
                    try
                    {
                        moved = await call.ResponseStream.MoveNext(ct);
                    }
                    catch (RpcException) when (!ct.IsCancellationRequested)
                    {
                        // Transport error mid-stream — reconnect and resume.
                        break;
                    }

                    if (!moved)
                    {
                        streamEnded = true;
                        break;
                    }

                    var env = call.ResponseStream.Current;
                    fromValue = (env.Offset?.Value ?? fromValue) + 1UL;
                    yield return env;
                }
            }

            // Stream closed (clean close or error) — reconnect to resume.
            await Task.Delay(backoff, ct);
            backoff = NextBackoff(backoff, p);
        }
    }

    private static TimeSpan NextBackoff(TimeSpan current, ReconnectPolicy p)
    {
        var next = TimeSpan.FromTicks(current.Ticks * p.Multiplier);
        return next < p.MaxBackoff ? next : p.MaxBackoff;
    }

    public void Dispose() => _channel.Dispose();
}

/// <summary>
/// Backoff/retry policy for a resilient subscription. Mirrors the Rust/TS/Go SDKs.
/// </summary>
public sealed record ReconnectPolicy(
    TimeSpan InitialBackoff,
    TimeSpan MaxBackoff,
    int Multiplier,
    int? MaxRetries)
{
    /// <summary>The standard policy (250ms → 30s, x2, retry forever).</summary>
    public static ReconnectPolicy Default { get; } = new(
        TimeSpan.FromMilliseconds(250),
        TimeSpan.FromSeconds(30),
        2,
        null);
}
