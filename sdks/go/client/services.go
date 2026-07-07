package client

import (
	"net/http"

	"connectrpc.com/connect"

	"github.com/prometheusags/frf/sdks/go/gen/flint/v1/flintv1connect"
)

// Thin clients for the gateway services beyond SpineService.
//
// All of these now have a live gateway server implementation: Sync (CRDT sync),
// Agent (agent runs), Signal (WebRTC signaling), Entity (read/watch, live since
// p17-c004), and Authz (relation check/write/delete, live since p17-c005).

// NewSyncClient constructs a SyncService client targeting baseURL.
func NewSyncClient(baseURL string, opts ...connect.ClientOption) flintv1connect.SyncServiceClient {
	return flintv1connect.NewSyncServiceClient(http.DefaultClient, baseURL, opts...)
}

// NewAgentClient constructs an AgentService client targeting baseURL.
func NewAgentClient(baseURL string, opts ...connect.ClientOption) flintv1connect.AgentServiceClient {
	return flintv1connect.NewAgentServiceClient(http.DefaultClient, baseURL, opts...)
}

// NewSignalClient constructs a SignalService client targeting baseURL.
func NewSignalClient(baseURL string, opts ...connect.ClientOption) flintv1connect.SignalServiceClient {
	return flintv1connect.NewSignalServiceClient(http.DefaultClient, baseURL, opts...)
}

// NewEntityClient constructs an EntityService client targeting baseURL.
func NewEntityClient(baseURL string, opts ...connect.ClientOption) flintv1connect.EntityServiceClient {
	return flintv1connect.NewEntityServiceClient(http.DefaultClient, baseURL, opts...)
}

// NewAuthzClient constructs an AuthzService client targeting baseURL.
func NewAuthzClient(baseURL string, opts ...connect.ClientOption) flintv1connect.AuthzServiceClient {
	return flintv1connect.NewAuthzServiceClient(http.DefaultClient, baseURL, opts...)
}
