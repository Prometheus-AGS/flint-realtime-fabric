using PrometheusAgs.Frf.Flint.V1;
using Grpc.Net.Client;

namespace PrometheusAgs.Frf.Flint.V1.Client;

/// <summary>
/// Factory wrappers for the gateway services beyond SpineService.
///
/// All of these now have a live gateway server implementation: Sync (CRDT sync),
/// Agent (agent runs), Signal (WebRTC signaling), Entity (read/watch, live since
/// p17-c004), and Authz (relation check/write/delete, live since p17-c005).
/// </summary>
public static class ServiceClients
{
    public static SyncService.SyncServiceClient CreateSyncClient(string address) =>
        new SyncService.SyncServiceClient(GrpcChannel.ForAddress(address));

    public static AgentService.AgentServiceClient CreateAgentClient(string address) =>
        new AgentService.AgentServiceClient(GrpcChannel.ForAddress(address));

    public static SignalService.SignalServiceClient CreateSignalClient(string address) =>
        new SignalService.SignalServiceClient(GrpcChannel.ForAddress(address));

    public static EntityService.EntityServiceClient CreateEntityClient(string address) =>
        new EntityService.EntityServiceClient(GrpcChannel.ForAddress(address));

    public static AuthzService.AuthzServiceClient CreateAuthzClient(string address) =>
        new AuthzService.AuthzServiceClient(GrpcChannel.ForAddress(address));
}
