/**
 * Browser-side decoded-media probe for the sovereign-SFU E2E harness (p23-c004).
 *
 * str0m (the sovereign SFU) is sans-codec — it forwards RTP, it never decodes frames. A real
 * decoded frame only exists inside the browser's WebRTC stack (VP8/H264/Opus → pixels/samples).
 * This probe therefore measures the decode where it actually happens: the receiving
 * `RTCPeerConnection`'s `getStats()` `inbound-rtp.framesDecoded` counter.
 *
 * It runs inside the page via Playwright `page.evaluate`. It connects a recvonly peer through
 * the gateway's `/ws/v1/signal` WebSocket (the c003 inbound path), waits for a track, and polls
 * getStats until a frame decodes or the timeout elapses. A non-zero `framesDecoded` is a genuine
 * end-to-end proof: signal → ICE → DTLS/SRTP → RTP relay by the SFU → browser decode.
 */

export interface DecodeProbeArgs {
  readonly wsUrl: string;
  readonly room: string;
  readonly tenant: string;
  readonly token?: string;
  readonly timeoutMs: number;
  /**
   * STUN server URL (e.g. `stun:127.0.0.1:3478`). When set, the browser gathers a
   * server-reflexive (`srflx`) candidate with a routable IP — str0m rejects the mDNS
   * `.local` host candidates Chrome emits by default (phase-28 B1 / p29-c002).
   */
  readonly stunUrl?: string;
  /**
   * TURN relay URL + credentials (e.g. `turn:coturn:3478`). When set, the browser gathers a
   * `typ relay` candidate carrying the relay server's real IP — the routable pair that
   * host/srflx candidates could not form on the bridge topology (phase-32). str0m accepts
   * `typ relay` (parser.rs:232). p34-c001.
   */
  readonly turnUrl?: string;
  readonly turnUsername?: string;
  readonly turnCredential?: string;
}

export interface DecodeProbeResult {
  readonly decoded: boolean;
  readonly framesDecoded: number;
  readonly bytesReceived: number;
  /** Terminal reason: `decoded`, `timeout`, `failed`, or `ws-error`. */
  readonly reason: string;
  /** Last observed `iceConnectionState` — tells you *where* a timeout stalled (p27-c001). */
  readonly lastIceState: string;
  /** Count of local candidates gathered + remote candidates received (p27-c001). */
  readonly localCandidates: number;
  readonly remoteCandidates: number;
}

export async function probeDecodedMedia(
  args: DecodeProbeArgs,
): Promise<DecodeProbeResult> {
  const { wsUrl, room, tenant, token, timeoutMs, stunUrl } = args;

  const tokenParam = token ? `&token=${encodeURIComponent(token)}` : "";
  const url =
    `${wsUrl}/ws/v1/signal?room=${encodeURIComponent(room)}` +
    `&tenant=${encodeURIComponent(tenant)}${tokenParam}`;

  // STUN gathers a `srflx` candidate; TURN gathers a `typ relay` candidate that carries the relay
  // server's real IP and always routes — the routable pair host/srflx couldn't form on the bridge
  // (phase-32 / p34-c001). str0m accepts `typ relay`. (This runs in the browser.)
  const iceServers: RTCIceServer[] = [];
  if (stunUrl) iceServers.push({ urls: stunUrl });
  if (args.turnUrl && args.turnUsername && args.turnCredential) {
    iceServers.push({
      urls: args.turnUrl,
      username: args.turnUsername,
      credential: args.turnCredential,
    });
  }
  // p36-c002d: force relay-only ICE when TURN is configured so the browser never gathers mDNS
  // host candidates (str0m rejects them — "bad address: invalid IP address syntax"). With relay-only
  // the browser skips the mDNS/srflx gather phase entirely and only produces `typ relay` candidates
  // carrying coturn's real bridge IP — str0m accepts those and ICE completes immediately.
  const hasTurn = !!(args.turnUrl && args.turnUsername && args.turnCredential);
  const pcConfig: RTCConfiguration =
    iceServers.length > 0
      ? { iceServers, ...(hasTurn ? { iceTransportPolicy: "relay" } : {}) }
      : {};
  const pc = new RTCPeerConnection(Object.keys(pcConfig).length > 0 ? pcConfig : undefined);
  pc.addTransceiver("video", { direction: "recvonly" });
  pc.addTransceiver("audio", { direction: "recvonly" });

  const socket = new WebSocket(url);

  /** Read the receiver's decoded-frame count from inbound-rtp stats. */
  const readStats = async (): Promise<{ frames: number; bytes: number }> => {
    let frames = 0;
    let bytes = 0;
    const stats = await pc.getStats();
    stats.forEach((report) => {
      if (report.type === "inbound-rtp") {
        const r = report as RTCInboundRtpStreamStats;
        frames += r.framesDecoded ?? 0;
        bytes += r.bytesReceived ?? 0;
      }
    });
    return { frames, bytes };
  };

  return new Promise<DecodeProbeResult>((resolve) => {
    const deadline = Date.now() + timeoutMs;
    // Diagnostics (p27-c001): track where a stalled run gets stuck.
    let lastIceState = pc.iceConnectionState;
    let localCandidates = 0;
    let remoteCandidates = 0;

    const settle = (partial: Omit<DecodeProbeResult, "lastIceState" | "localCandidates" | "remoteCandidates">) => {
      try {
        socket.close();
        pc.close();
      } catch {
        // best-effort teardown
      }
      resolve({ ...partial, lastIceState, localCandidates, remoteCandidates });
    };

    const poll = async () => {
      const { frames, bytes } = await readStats();
      if (frames > 0) {
        settle({ decoded: true, framesDecoded: frames, bytesReceived: bytes, reason: "decoded" });
        return;
      }
      if (Date.now() >= deadline) {
        settle({ decoded: false, framesDecoded: frames, bytesReceived: bytes, reason: "timeout" });
        return;
      }
      setTimeout(() => void poll(), 250);
    };

    pc.onicecandidate = (e) => {
      if (e.candidate) {
        localCandidates += 1;
        // p27-c002: trickle our candidate up to the SFU so it (the answerer) can complete ICE.
        socket.send(
          JSON.stringify({
            fromSession: "browser-rx",
            tenantId: tenant,
            roomId: room,
            kind: "ice-candidate",
            sfuMode: "sovereign",
            payload: { candidate: e.candidate.candidate },
          }),
        );
      }
    };
    pc.oniceconnectionstatechange = () => {
      lastIceState = pc.iceConnectionState;
      if (pc.iceConnectionState === "failed") {
        settle({ decoded: false, framesDecoded: 0, bytesReceived: 0, reason: "failed" });
      }
    };

    socket.onopen = async () => {
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);
      socket.send(
        JSON.stringify({
          fromSession: "browser-rx",
          tenantId: tenant,
          roomId: room,
          kind: "offer",
          sfuMode: "sovereign",
          payload: { sdp: offer.sdp },
        }),
      );
      void poll();
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
        // p27-c003: join the shared room so the SFU (RoomRouter) fans the sender's RTP to us.
        // Authorized by the ADR-007 Keto `view` grant seeded for this subject.
        socket.send(
          JSON.stringify({
            fromSession: "browser-rx",
            tenantId: tenant,
            roomId: room,
            kind: "room-join",
            sfuMode: "sovereign",
            payload: {},
          }),
        );
      } else if (frame.kind === "ice-candidate" && frame.payload?.candidate) {
        // p27-c002: apply the SFU's trickle candidate so the browser can reach it.
        remoteCandidates += 1;
        try {
          await pc.addIceCandidate({ candidate: frame.payload.candidate });
        } catch {
          // ignore malformed/duplicate candidate
        }
      }
    };

    socket.onerror = () =>
      settle({ decoded: false, framesDecoded: 0, bytesReceived: 0, reason: "ws-error" });
  });
}
