package client

import (
	"context"
	"fmt"
	"net/http"
	"time"

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

// ReconnectPolicy controls backoff and retry for a resilient subscription.
// Semantics mirror the Rust and TS SDKs.
type ReconnectPolicy struct {
	InitialBackoff time.Duration
	MaxBackoff     time.Duration
	Multiplier     int
	// MaxRetries is the max consecutive failed reconnects before giving up.
	// Zero means retry forever.
	MaxRetries int
}

// DefaultReconnectPolicy returns the standard policy (250ms→30s, x2, forever).
func DefaultReconnectPolicy() ReconnectPolicy {
	return ReconnectPolicy{
		InitialBackoff: 250 * time.Millisecond,
		MaxBackoff:     30 * time.Second,
		Multiplier:     2,
		MaxRetries:     0,
	}
}

// SubscribeResilient opens a reconnecting subscription. On a transport error or a
// cleanly-closed stream (e.g. a gateway restart) it reconnects with exponential
// backoff and resumes from last-seen-offset+1, so no event is skipped or replayed.
//
// Events are delivered on the returned channel, which is closed when ctx is
// cancelled or reconnection is exhausted. A non-nil error is sent on errc before
// the channel closes if reconnection was exhausted.
func (c *SpineClient) SubscribeResilient(
	ctx context.Context,
	req *flintv1.SubscribeRequest,
	policy ReconnectPolicy,
) (<-chan *flintv1.EventEnvelope, <-chan error) {
	events := make(chan *flintv1.EventEnvelope)
	errc := make(chan error, 1)

	go func() {
		defer close(events)
		defer close(errc)

		fromValue := req.GetFrom().GetValue()
		backoff := policy.InitialBackoff
		retries := 0

		for {
			if ctx.Err() != nil {
				return
			}

			attempt := &flintv1.SubscribeRequest{
				ChannelId:  req.GetChannelId(),
				ConsumerId: req.GetConsumerId(),
				From:       &flintv1.Offset{Value: fromValue},
			}

			stream, err := c.inner.Subscribe(ctx, connect.NewRequest(attempt))
			if err == nil {
				backoff = policy.InitialBackoff
				retries = 0
				for stream.Receive() {
					env := stream.Msg()
					// Resume AFTER the last delivered offset.
					fromValue = env.GetOffset().GetValue() + 1
					select {
					case events <- env:
					case <-ctx.Done():
						_ = stream.Close()
						return
					}
				}
				_ = stream.Close()
				// Stream ended (clean close or recv error) — reconnect to resume.
			} else {
				retries++
				if policy.MaxRetries > 0 && retries > policy.MaxRetries {
					errc <- fmt.Errorf("reconnection exhausted after %d attempts: %w", retries, err)
					return
				}
			}

			select {
			case <-time.After(backoff):
			case <-ctx.Done():
				return
			}
			backoff = minDuration(backoff*time.Duration(policy.Multiplier), policy.MaxBackoff)
		}
	}()

	return events, errc
}

func minDuration(a, b time.Duration) time.Duration {
	if a < b {
		return a
	}
	return b
}
