## ADDED Requirements

### Requirement: Versioned routed-observer records preserve distinct identities

Fabric SHALL accept a routed-observer record only when it carries profile `frf.routed-observer/1`, stable source occurrence and provider-native message identities, provider/account/workspace/room/thread/sender scope, tenant, selected handler and route, route/binding/policy revisions, original principal, observer and delivery identities, payload classification, and causal root/parent/action/depth/fanout/visited routes. The subscriber cursor identity SHALL remain separate from Fabric's transport offset and Iggy consumer position.

#### Scenario: Routed copy crosses the frozen transport

- **WHEN** a complete profile is wrapped in an `EventEnvelope`, published, read from Iggy, and carried through protobuf v1 custom-event conversion
- **THEN** its JSON profile discriminator and semantic identities remain available for strict decoding, even though protobuf v1 does not preserve the custom-kind string

#### Scenario: Legacy or incomplete record is offered to a routed consumer

- **WHEN** a consumer requests the routed profile from an ordinary envelope, an unknown profile version, or a record missing required identities
- **THEN** decoding reports unsupported or invalid profile rather than synthesizing source, recipient, or cursor identities

### Requirement: Transport admission preserves authorization boundaries

Fabric SHALL reject a routed profile whose source tenant differs from the enclosing channel, whose correlation differs from the stable source occurrence, or whose causal trace exceeds the version-one ceiling of depth four and cumulative fanout eight. Profile decoding SHALL NOT grant observation, execution, or reply authority; gateway publish and subscribe flows SHALL continue to apply their existing authentication and per-event policy checks.

#### Scenario: Cross-tenant copy is published

- **WHEN** a routed observer delivery names a source tenant different from its channel tenant
- **THEN** the record is rejected before broker admission

#### Scenario: Observer is revoked before delivery

- **WHEN** a previously admitted record is replayed to a subscriber whose current grant is absent
- **THEN** the resource-side policy check withholds delivery; possession of the transport record does not authorize disclosure or execution

### Requirement: Runtime cancellation is not inferred from stream detachment

Fabric SHALL report `AgentRunControl.cancel` as unsupported runtime control through the current bidirectional stream and close only that observer stream. It SHALL NOT report that the agent run was cancelled.

#### Scenario: Client requests cancellation through Fabric

- **WHEN** a client sends a cancel control frame on `RunAgent`
- **THEN** the stream terminates with an explicit unsupported status and the runtime owner retains control of the run
