#!/usr/bin/env node
// mint-e2e-jwt.mjs (p25-c001) — mint an RS256 JWT the gateway accepts, and emit its JWKS.
//
// The gateway verifier (frf-identity-ory) is hardcoded to RS256 + JWKS public keys
// (verifier.rs), so a flint-gate HS256 token would be rejected. This mints a real RS256 token
// with the claims FrfClaims requires (sub, tenant_id, jti, aud, exp) and writes the matching
// public JWK set — point GATEWAY_JWKS_URL at a server for that file and the gateway verifies it.
//
// Zero dependencies — uses node's built-in `crypto`.
//
// Usage:
//   node scripts/mint-e2e-jwt.mjs --out-dir <dir> --sub user:e2e --aud frf-gateway \
//        --tenant 00000000-0000-0000-0000-000000000000
// Writes: <dir>/jwt.txt (the token) and <dir>/jwks.json (the public key set).

import { generateKeyPairSync, createSign, randomUUID } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

function arg(name, fallback) {
  const i = process.argv.indexOf(`--${name}`);
  return i !== -1 && process.argv[i + 1] ? process.argv[i + 1] : fallback;
}

const outDir = resolve(arg("out-dir", "./.e2e-jwt"));
const sub = arg("sub", "user:e2e");
const aud = arg("aud", "frf-gateway");
const iss = arg("iss", "frf-e2e"); // must match the gateway's JWT_ISSUER (p26-c002)
const tenant = arg("tenant", "00000000-0000-0000-0000-000000000000");
const ttlSeconds = Number(arg("ttl", "3600"));
const kid = "e2e-rs256";

const base64url = (buf) =>
  Buffer.from(buf).toString("base64").replace(/=/g, "").replace(/\+/g, "-").replace(/\//g, "_");

// 1. RSA keypair.
const { publicKey, privateKey } = generateKeyPairSync("rsa", { modulusLength: 2048 });

// 2. JWT header + claims (FrfClaims: sub, tenant_id, jti, aud, exp).
const now = Math.floor(Date.now() / 1000);
const header = { alg: "RS256", typ: "JWT", kid };
const claims = {
  sub,
  tenant_id: tenant,
  jti: randomUUID(),
  aud,
  iss,
  iat: now,
  exp: now + ttlSeconds,
};

const signingInput = `${base64url(JSON.stringify(header))}.${base64url(JSON.stringify(claims))}`;
const signature = createSign("RSA-SHA256").update(signingInput).sign(privateKey);
const jwt = `${signingInput}.${base64url(signature)}`;

// 3. Public JWK set (add kid + RS256 use so the verifier's JWKS lookup matches).
const jwk = publicKey.export({ format: "jwk" });
const jwks = { keys: [{ ...jwk, kid, alg: "RS256", use: "sig" }] };

mkdirSync(outDir, { recursive: true });
writeFileSync(resolve(outDir, "jwt.txt"), jwt, "utf8");
writeFileSync(resolve(outDir, "jwks.json"), JSON.stringify(jwks), "utf8");

// Print the token to stdout so callers can capture it.
process.stdout.write(jwt);
