//go:build integration

// Smoke-test subscriber for the Go SDK.
// Usage: FRF_GATEWAY_URL=http://localhost:4000 go run -tags integration ./tests/e2e/go/
//
// Exits 0 when it receives at least one EventEnvelope within the timeout.
package main

import (
	"context"
	"fmt"
	"net/http"
	"os"
	"time"

	"github.com/prometheusags/frf/sdks/go/client"
	flintv1 "github.com/prometheusags/frf/sdks/go/gen/flint/v1"
)

func main() {
	gatewayURL := os.Getenv("FRF_GATEWAY_URL")
	if gatewayURL == "" {
		gatewayURL = "http://localhost:4000"
	}

	channelID := os.Getenv("FRF_CHANNEL_ID")
	if channelID == "" {
		channelID = "00000000-0000-0000-0000-000000000001"
	}

	c := client.New(gatewayURL, client.WithHTTPClient(&http.Client{}))

	ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
	defer cancel()

	req := &flintv1.SubscribeRequest{
		ChannelId:  channelID,
		ConsumerId: "e2e-go-smoke",
	}

	stream, err := c.Subscribe(ctx, req)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Subscribe error: %v\n", err)
		os.Exit(1)
	}

	fmt.Println("Subscribed; waiting for event…")
	msg, err := stream.Receive()
	if err != nil {
		fmt.Fprintf(os.Stderr, "Receive error: %v\n", err)
		os.Exit(1)
	}

	fmt.Printf("OK — received envelope id=%s kind=%v\n", msg.GetId(), msg.GetKind())
}
