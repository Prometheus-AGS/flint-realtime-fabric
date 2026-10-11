#!/usr/bin/env node
import { createSign, generateKeyPairSync, randomUUID } from "node:crypto";
import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

function requiredArg(name) {
  const index = process.argv.indexOf(`--${name}`);
  if (index === -1 || !process.argv[index + 1]) {
    throw new Error(`--${name} is required`);
  }
  return process.argv[index + 1];
}

const output = resolve(requiredArg("out-dir"));
const tenant = requiredArg("tenant");
const crossTenant = requiredArg("cross-tenant");
const issuer = requiredArg("issuer");
const audience = requiredArg("audience");
const base64url = (value) => Buffer.from(value).toString("base64url");

const idp = generateKeyPairSync("rsa", { modulusLength: 2048 });
const gate = generateKeyPairSync("rsa", { modulusLength: 2048 });
const idpKid = "pri-c005-idp";

function token(subject, tenantId) {
  const now = Math.floor(Date.now() / 1000);
  const header = { alg: "RS256", typ: "JWT", kid: idpKid };
  const claims = {
    sub: subject,
    tenant_id: tenantId,
    aud: audience,
    iss: issuer,
    iat: now,
    exp: now + 600,
    jti: randomUUID(),
  };
  const input = `${base64url(JSON.stringify(header))}.${base64url(JSON.stringify(claims))}`;
  const signature = createSign("RSA-SHA256").update(input).sign(idp.privateKey);
  return `${input}.${base64url(signature)}`;
}

mkdirSync(output, { recursive: true });
const idpJwk = idp.publicKey.export({ format: "jwk" });
writeFileSync(
  resolve(output, "idp-jwks.json"),
  `${JSON.stringify({ keys: [{ ...idpJwk, kid: idpKid, alg: "RS256", use: "sig" }] })}\n`,
);
writeFileSync(resolve(output, "allowed-inbound.txt"), token("user:allowed", tenant));
writeFileSync(resolve(output, "denied-inbound.txt"), token("user:denied", tenant));
writeFileSync(resolve(output, "cross-inbound.txt"), token("user:cross", crossTenant));
writeFileSync(
  resolve(output, "gate-private.pem"),
  gate.privateKey.export({ type: "pkcs8", format: "pem" }),
  { mode: 0o600 },
);
writeFileSync(
  resolve(output, "gate-public.pem"),
  gate.publicKey.export({ type: "spki", format: "pem" }),
);
