//go:build integration

package client_test

import (
	"context"
	"os"
	"testing"

	flintv1 "github.com/prometheusags/frf/sdks/go/gen/flint/v1"
)

func TestPublishSmoke(t *testing.T) {
	baseURL := os.Getenv("FRF_GATEWAY_URL")
	if baseURL == "" {
		t.Skip("FRF_GATEWAY_URL not set")
	}

	c := New(baseURL)
	_, err := c.Publish(context.Background(), &flintv1.EventEnvelope{})
	if err != nil {
		t.Logf("publish returned (expected error without auth): %v", err)
	}
}
