# SSR restricted-shape profile provenance

This overlay is a render template for the accepted `shape-only` deployment
boundary. Direct application intentionally fails because `gateway.yaml` carries
the zero image digest and `ingress.yaml` carries the reserved `fabric.invalid`
host. Render with:

```bash
FRF_SHAPE_GATEWAY_IMAGE='registry.example/frf-shape@sha256:<digest>' \
FRF_PUBLIC_HOST='fabric.example.com' \
FRF_TLS_SECRET_NAME='frf-shape-tls' \
FRF_TLS_CERT_FILE='/run/secrets/tls.crt' \
FRF_TLS_KEY_FILE='/run/secrets/tls.key' \
FRF_TLS_CA_FILE='/run/secrets/ca.crt' \
k8s/overlays/ssr/render-profile.sh > rendered.yaml
```

The renderer accepts only an immutable nonzero image digest, a real DNS host,
and a valid trusted certificate with its matching key. It emits the named TLS
Secret, so the rendered output contains secret material and must be protected.
The image must be built from the repository `Dockerfile` with
`CARGO_FEATURES=shape-facade`; the gateway validates that feature at startup.

## Authority and network boundary

- The public Ingress terminates TLS from the rendered Secret and forces HTTPS.
- Ingress sends public traffic to `flint-gate-proxy`; it does not expose the Fabric
  Service as a backend route.
- The NetworkPolicy admits gateway HTTP only from Gate.
- The gateway reaches Gate JWKS and Electric on their internal Services.
- The shape catalog comes from ConfigMap `aso-shape-catalog` and is mounted
  read-only.
- gRPC, Iggy, CDC, federation, media and the general event surface are absent.
- Gateway readiness performs real Gate JWKS and Electric health requests.

Gate, Electric, Postgres, shape-catalog ConfigMap and ingress
controller are deployment-owned prerequisites. Their exact revisions and
configuration digests belong in the environment's release receipt; this source
template does not invent them.
