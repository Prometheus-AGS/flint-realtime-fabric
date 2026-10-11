# c010 adversarial review round 20 resolution

## Warning: checkpoint key could register an inert watch service without CDC

Disposition: fixed. The gRPC runtime now registers the v2 entity-type watch
service only when `CDC_ENABLED=true`. That branch consumes the checkpoint key,
source epoch, and enrollment JSON guaranteed by successful CDC validation; it
no longer invents a `disabled` epoch or an empty enrollment set. When CDC is
off, the runtime logs that v2 watch is disabled and omits the service even if an
unused checkpoint-key variable is present.

Strict all-target gateway Clippy and all 59 gateway library tests pass.
