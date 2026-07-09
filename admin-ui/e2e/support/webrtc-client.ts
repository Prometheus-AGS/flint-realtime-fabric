/**
 * Browser-side WebRTC client for the sovereign-SFU E2E harness (p23-c003).
 *
 * Runs inside the page via Playwright's `page.evaluate` — it drives a real
 * `RTCPeerConnection` against the gateway's `/ws/v1/signal` WebSocket:
 *
 *   offer (SDP) --WS--> gateway --> MediaTransportBridge --> Answer (SDP) --WS--> here
 *
 * and resolves once ICE reaches `connected`. This is the *real* browser transport a
 * user's client uses — not a gRPC harness — so a pass here is a genuine end-to-end
 * signal+DTLS/ICE proof against the sovereign gateway.
 *
 * It carries no Node/Playwright imports so it can be serialized into the page. The SDP
 * frame shape mirrors the gateway `SignalFrame` / `InboundSignalFrame` (camelCase).
 */

/** Result surfaced back to the Playwright test. */
export interface RtcConnectResult {
  readonly connected: boolean;
  /** Terminal `iceConnectionState` observed (`connected`, `failed`, or `timeout`). */
  readonly state: string;
}

/** Args passed from the Node test into the browser context. */
export interface RtcConnectArgs {
  readonly wsUrl: string;
  readonly room: string;
  readonly tenant: string;
  readonly token?: string;
  /** Milliseconds to wait for `connected` before giving up. */
  readonly timeoutMs: number;
}

/**
 * The function body below is what runs *in the browser*. Export it as a string-safe
 * function so a test can do `page.evaluate(connectToSovereignSfu, args)`.
 */
export async function connectToSovereignSfu(
  args: RtcConnectArgs,
): Promise<RtcConnectResult> {
  const { wsUrl, room, tenant, token, timeoutMs } = args;

  const tokenParam = token ? `&token=${encodeURIComponent(token)}` : "";
  const url =
    `${wsUrl}/ws/v1/signal?room=${encodeURIComponent(room)}` +
    `&tenant=${encodeURIComponent(tenant)}${tokenParam}`;

  const pc = new RTCPeerConnection();
  // A recv-only audio transceiver is enough to force a real m-line + ICE/DTLS.
  pc.addTransceiver("audio", { direction: "recvonly" });

  const socket = new WebSocket(url);

  const done = new Promise<RtcConnectResult>((resolve) => {
    const settle = (connected: boolean, state: string) => {
      try {
        socket.close();
        pc.close();
      } catch {
        // best-effort teardown
      }
      resolve({ connected, state });
    };

    const timer = setTimeout(() => settle(false, "timeout"), timeoutMs);

    pc.oniceconnectionstatechange = () => {
      const s = pc.iceConnectionState;
      if (s === "connected" || s === "completed") {
        clearTimeout(timer);
        settle(true, "connected");
      } else if (s === "failed" || s === "closed") {
        clearTimeout(timer);
        settle(false, s);
      }
    };

    pc.onicecandidate = (e) => {
      // p27-c002: trickle our candidate up to the SFU (the answerer) so ICE can complete.
      if (e.candidate) {
        socket.send(
          JSON.stringify({
            fromSession: "browser",
            tenantId: tenant,
            roomId: room,
            kind: "ice-candidate",
            sfuMode: "sovereign",
            payload: { candidate: e.candidate.candidate },
          }),
        );
      }
    };

    socket.onopen = async () => {
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);
      socket.send(
        JSON.stringify({
          fromSession: "browser",
          tenantId: tenant,
          roomId: room,
          kind: "offer",
          sfuMode: "sovereign",
          payload: { sdp: offer.sdp },
        }),
      );
    };

    socket.onmessage = async (event: MessageEvent<string>) => {
      let frame: { kind?: string; payload?: { sdp?: string; candidate?: string } };
      try {
        frame = JSON.parse(event.data) as typeof frame;
      } catch {
        return;
      }
      if (frame.kind === "answer" && frame.payload?.sdp) {
        await pc.setRemoteDescription({ type: "answer", sdp: frame.payload.sdp });
      } else if (frame.kind === "ice-candidate" && frame.payload?.candidate) {
        try {
          await pc.addIceCandidate({ candidate: frame.payload.candidate });
        } catch {
          // ignore malformed/duplicate candidate
        }
      }
    };

    socket.onerror = () => settle(false, "ws-error");
  });

  return done;
}
