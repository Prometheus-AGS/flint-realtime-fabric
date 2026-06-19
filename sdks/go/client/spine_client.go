package client

import (
	"context"
	"net/http"

	"connectrpc.com/connect"

	flintv1 "github.com/prometheusags/frf/sdks/go/gen/flint/v1"
	"github.com/prometheusags/frf/sdks/go/gen/flint/v1/flintv1connect"
)

// SpineClient wraps the generated Connect client for flint.v1.SpineService.
type SpineClient struct {
	inner flintv1connect.SpineServiceClient
}

// New constructs a SpineClient targeting baseURL.
// Callers may pass connect.ClientOption values to configure TLS, interceptors, etc.
func New(baseURL string, opts ...connect.ClientOption) *SpineClient {
	return &SpineClient{
		inner: flintv1connect.NewSpineServiceClient(http.DefaultClient, baseURL, opts...),
	}
}

// Publish publishes an EventEnvelope to the spine.
func (c *SpineClient) Publish(ctx context.Context, envelope *flintv1.EventEnvelope) (*flintv1.PublishResponse, error) {
	req := connect.NewRequest(&flintv1.PublishRequest{Envelope: envelope})
	resp, err := c.inner.Publish(ctx, req)
	if err != nil {
		return nil, err
	}
	return resp.Msg, nil
}

// Subscribe opens a server-streaming subscription for the given request.
// The returned stream must be closed by the caller.
func (c *SpineClient) Subscribe(ctx context.Context, req *flintv1.SubscribeRequest) (*connect.ServerStreamForClient[flintv1.EventEnvelope], error) {
	return c.inner.Subscribe(ctx, connect.NewRequest(req))
}
