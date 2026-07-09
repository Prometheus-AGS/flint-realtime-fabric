/**
 * Media E2E (G2): a receiver DECODES media relayed by the sovereign SFU (p23-c004).
 *
 * WHY THIS IS BROWSER-SIDE: str0m (the SFU) is sans-codec — it forwards RTP, it never decodes
 * frames. A real decoded frame only exists inside a browser's WebRTC stack. So the honest proof
 * measures the decode where it happens: the receiving peer's `getStats()`
 * `inbound-rtp.framesDecoded > 0`. A sender peer publishes a fake camera track through the
 * sovereign gateway; the receiver peer decodes it.
 *
 * HONEST GATE — never a false green:
 *   - `test.skip`-gated on SKIP_INTEGRATION / GATEWAY_URL. With no live SFU_MODE=sovereign
 *     gateway (and no Chromium fake-media), it is SKIPPED, not passed.
 *   - This headless CI environment has neither, so this test does not run here. The harness is
 *     proven-capable, not proven-here — which is why the phase keeps SFU_MODE=sovereign OFF
 *     (p23-c006 re-affirms gated).
 *
 * To run: launch a gateway with SFU_MODE=sovereign, set GATEWAY_URL + SKIP_INTEGRATION=false,
 * and run Chromium with fake media (browserContext args below).
 */

import { test, expect, chromium } from "@playwright/test";
import { probeDecodedMedia } from "./support/decode-probe.js";

const skipIntegration =
  process.env["SKIP_INTEGRATION"] === "true" || !process.env["GATEWAY_URL"];
const GATEWAY_URL = process.env["GATEWAY_URL"] ?? "http://localhost:28080";
const WS_URL = GATEWAY_URL.replace(/^http/, "ws");
const TENANT = process.env["E2E_TENANT_ID"] ?? "00000000-0000-0000-0000-000000000000";
const TOKEN = process.env["E2E_JWT"];
const ROOM = "e2e-decode-room";
// STUN so the browser gathers a routable `srflx` candidate (phase-28 B1 / p29-c002). Default to
// the hermetic local coturn from compose.sovereign.yml; override with STUN_URL.
const STUN_URL = process.env["STUN_URL"] ?? "stun:127.0.0.1:3478";
// TURN relay (p34-c001): a `typ relay` candidate carries the relay server's real IP and always
// routes — the routable pair host/srflx couldn't form on the bridge (phase-32). Credentials come
// from env (never committed); TURN is added only when TURN_URL + creds are all present.
const TURN_URL = process.env["TURN_URL"];
const TURN_USERNAME = process.env["TURN_USERNAME"];
const TURN_CREDENTIAL = process.env["TURN_CREDENTIAL"];

/**
 * Chromium launch args for both the sender and receiver contexts. The decode stack now fronts the
 * gateway with a TLS sidecar (p32-c001), so `GATEWAY_URL` is `https://caddy:8443` — a **genuine
 * secure context** where `navigator.mediaDevices` / `getUserMedia` / `getStats` work with NO unsafe
 * flags (the p31 `--unsafely-treat-insecure-origin-as-secure-origin` combination was unreliable in
 * the headless container and is removed). For the self-signed internal cert, add
 * `--ignore-certificate-errors`; the `localhost`/`https` host-run path needs no cert flag.
 */
function chromiumArgs(): string[] {
  const args = [
    "--use-fake-ui-for-media-stream",
    "--use-fake-device-for-media-stream",
  ];
  // Trust the TLS sidecar's self-signed internal cert (only an in-network `https://caddy:…` origin;
  // a real `localhost` host run uses a trusted/loopback context and needs no override).
  const origin = new URL(GATEWAY_URL).origin;
  const isLoopback = /^https?:\/\/(localhost|127\.0\.0\.1)/.test(origin);
  if (/^https:/.test(origin) && !isLoopback) {
    args.push("--ignore-certificate-errors");
  }
  return args;
}

test.describe("Media: receiver decodes SFU-relayed media (requires gateway + Chromium media)", () => {
  test.skip(
    skipIntegration,
    "Set SKIP_INTEGRATION=false + GATEWAY_URL to a running SFU_MODE=sovereign gateway. " +
      "str0m is sans-codec: the decode is browser-side, so this needs Chromium fake-media.",
  );

  test("receiver reports inbound-rtp.framesDecoded > 0", async () => {
    // p28-c001: raise the per-test timeout above the probe's own 20s deadline so the probe
    // settles-with-diagnostics and the assertion message prints (default 30s aborted mid-probe).
    test.setTimeout(60_000);
    // A dedicated fake-media Chromium context is the *sender* — it publishes a synthetic
    // camera track into the room. The default test context hosts the *receiver* probe.
    const senderCtx = await chromium.launchPersistentContext("", {
      args: chromiumArgs(),
    });
    try {
      const senderPage = await senderCtx.newPage();
      // Navigate to the gateway origin (a localhost secure context) — `navigator.mediaDevices`
      // is undefined on `about:blank` (insecure context), which breaks getUserMedia.
      await senderPage.goto(`${GATEWAY_URL}/`);
      // Sender: publish a fake camera track through the gateway.
      await senderPage.evaluate(async (args) => {
        const media = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
        // Reuse the shared connect helper's transport shape inline: send an offer that includes
        // the fake track so the SFU relays real RTP. STUN gathers a srflx candidate; TURN gathers a
        // `typ relay` candidate that always routes (phase-32 had no routable pair; p34-c001).
        const iceServers: RTCIceServer[] = [];
        if (args.stunUrl) iceServers.push({ urls: args.stunUrl });
        if (args.turnUrl && args.turnUsername && args.turnCredential) {
          iceServers.push({
            urls: args.turnUrl,
            username: args.turnUsername,
            credential: args.turnCredential,
          });
        }
        const pc = new RTCPeerConnection(
          iceServers.length > 0 ? { iceServers } : undefined,
        );
        media.getTracks().forEach((t) => pc.addTrack(t, media));
        const url =
          `${args.wsUrl}/ws/v1/signal?room=${encodeURIComponent(args.room)}` +
          `&tenant=${encodeURIComponent(args.tenant)}` +
          (args.token ? `&token=${encodeURIComponent(args.token)}` : "");
        const socket = new WebSocket(url);
        await new Promise<void>((resolve) => {
          socket.onopen = async () => {
            const offer = await pc.createOffer();
            await pc.setLocalDescription(offer);
            socket.send(
              JSON.stringify({
                fromSession: "browser-tx",
                tenantId: args.tenant,
                roomId: args.room,
                kind: "offer",
                sfuMode: "sovereign",
                payload: { sdp: offer.sdp },
              }),
            );
            resolve();
          };
          // p27-c002: trickle the sender's candidates up to the SFU so ICE completes.
          pc.onicecandidate = (e) => {
            if (e.candidate) {
              socket.send(
                JSON.stringify({
                  fromSession: "browser-tx",
                  tenantId: args.tenant,
                  roomId: args.room,
                  kind: "ice-candidate",
                  sfuMode: "sovereign",
                  payload: { candidate: e.candidate.candidate },
                }),
              );
            }
          };
          socket.onmessage = async (event: MessageEvent<string>) => {
            const frame = JSON.parse(event.data) as {
              kind?: string;
              payload?: { sdp?: string; candidate?: string };
            };
            if (frame.kind === "answer" && frame.payload?.sdp) {
              await pc.setRemoteDescription({ type: "answer", sdp: frame.payload.sdp });
              // p27-c003: join the shared room so the SFU fans this sender's RTP to the receiver.
              socket.send(
                JSON.stringify({
                  fromSession: "browser-tx",
                  tenantId: args.tenant,
                  roomId: args.room,
                  kind: "room-join",
                  sfuMode: "sovereign",
                  payload: {},
                }),
              );
            } else if (frame.kind === "ice-candidate" && frame.payload?.candidate) {
              try {
                await pc.addIceCandidate({ candidate: frame.payload.candidate });
              } catch {
                // ignore malformed/duplicate
              }
            }
          };
        });
      }, { wsUrl: WS_URL, room: ROOM, tenant: TENANT, token: TOKEN, stunUrl: STUN_URL,
           turnUrl: TURN_URL, turnUsername: TURN_USERNAME, turnCredential: TURN_CREDENTIAL });

      // Receiver: connect and poll getStats for a decoded frame.
      const receiverCtx = await chromium.launchPersistentContext("", {
        args: chromiumArgs(),
      });
      try {
        const rxPage = await receiverCtx.newPage();
        // Gateway origin = secure context (localhost), so RTCPeerConnection/getStats work.
        await rxPage.goto(`${GATEWAY_URL}/`);
        // p28-c001: the probe self-connects (offer/answer/ICE/RoomJoin) and asserts decode. The
        // former `connectToSovereignSfu` pre-check was a redundant separate session that burned 15s
        // of the test budget (pushing connect+probe past Playwright's 30s timeout and masking the
        // diagnostics). Dropped — the probe is the single source of truth.
        const result = await rxPage.evaluate(probeDecodedMedia, {
          wsUrl: WS_URL,
          room: ROOM,
          tenant: TENANT,
          token: TOKEN,
          timeoutMs: 20_000,
          stunUrl: STUN_URL,
          turnUrl: TURN_URL,
          turnUsername: TURN_USERNAME,
          turnCredential: TURN_CREDENTIAL,
        });
        expect(
          result.decoded,
          `framesDecoded=${result.framesDecoded} bytes=${result.bytesReceived} reason=${result.reason} ` +
            `ice=${result.lastIceState} localCandidates=${result.localCandidates} remoteCandidates=${result.remoteCandidates}`,
        ).toBeTruthy();
      } finally {
        await receiverCtx.close();
      }
    } finally {
      await senderCtx.close();
    }
  });
});
