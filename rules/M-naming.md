# M — Naming

A name tells you the role. If you can't tell from the name whether something
is a port, an adapter, or a wrapper, the name is wrong.

| Thing | Pattern |
|---|---|
| Value object | `<Entity><Attribute>` (`EntityId`, `EntityName`) |
| Domain error | `<Entity>Error`, `<Slice>Error` |
| Command | `Update<Entity>`, `Create<Entity>` |
| Projection | `<Entity>Detail`, `<Entity>Summary` |
| Service | `<Capability>Service` (`FleetService`, `EnvService`) |
| Service error | `<Capability>ServiceError` |
| Port | `<Capability>Repository` (`EntityRepository`, `EnvRepository`) or a capability noun (`Logger`, `Clock`) |
| Dispatch wrapper | `<Port>Repo` (`EntityRepo`, `EnvRepo`, `LoggerRepo`) — never `App<X>` |
| Adapter | `<Vendor><Port>` (`SqlxEntityRepository`, `LocalEnvRepository`, `TracingLogger`) |
| Catalog | `<Capability>Option` (`EnvOption`) + `<Capability>SourceKind` |
| Catalog metadata | `<Catalog>Meta` (`EnvOptionMeta`) |
| Request | `<Action><Entity>Request` |
| Response | `<Entity><Action>Response`, one per action |
| Edge error | `<Entity><Action>Error`, one per action |
| Validation | `<Entity>ValidationErrors` |
| Limits | `<TYPE>_<BOUND>` constants (`ENTITY_ID_MAX_LENGTH`) |
| Bootstrap error | `BootstrapError` |
| Generator | lower-case pattern name (`dispatch`, `catalog`) |

## Rules

### M1 — Names follow the table
**Check:** every type in a role listed above matches its pattern.

### M2 — Files are named for their single type or role
`<vendor>_<entity>_repository`, `<entity>_repo`, `<capability>_service`
(A1, S2).
