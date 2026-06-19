using PrometheusAgs.Frf.Flint.V1;
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

    public void Dispose() => _channel.Dispose();
}
