# V — Services

The service is the application's API to itself. Every driving edge (HTTP,
CLI, cron, queue) calls the same services, and none of them knows which.

## Rules

### V1 — Services are the only thing the edge talks to
They own orchestration: multi-step operations, cascades, cross-slice
composition.

**Check:** edge code never imports a repository, adapter, or port.

### V2 — Validated types in, validated types out
Services take and return validated domain types — never raw primitives,
never request/response types. Field validity is guaranteed upstream by the
edge's conversion (E3); services enforce *relational* rules only (referenced
entity exists, no duplicate assignment).

**Check:** no service re-validates a field; no service signature mentions a
wire type.

### V3 — Read aggregates are projection types
Read-only aggregates are projection structs in the domain (`EntityDetail`),
distinct from entities, built by the service.

**Check:** no entity grows optional fields to carry joined data.

### V4 — Service errors compose slice errors
A service error wraps its slices' errors transparently, with automatic
conversion, and adds its own variants for orchestration-level failures.

**Check:** no manual re-mapping of a slice error into a near-identical
service variant.

### V5 — Services depend on ports, via dispatch wrappers
Never on concrete adapters.

**Check:** service fields are dispatch-wrapper types (`EntityRepo`).

### V6 — Capability slices get services too
When there is orchestration to own. Routing catalog options across multiple
live sources (`EnvService`) is a service job exactly like cross-entity
composition (`FleetService`). The dispatch wrapper answers *which adapter
implements this slot*; the service answers *which slot owns this value*.

**Check:** source-routing logic lives in a service, not in bootstrap or a
dispatch wrapper.

## Pseudocode

```
service AssignmentService
  holds: entities: EntityRepo, teams: TeamRepo

  assign(command: AssignEntity) -> Assignment | AssignmentServiceError
    entity = entities.fetch(command.entity_id())?        # EntityError -> transparent (V4)
      or AssignmentServiceError.EntityNotFound(command.entity_id())
    team   = teams.fetch(command.team_id())?
      or AssignmentServiceError.TeamNotFound(command.team_id())
    if team.has(entity.id()): return AssignmentServiceError.AlreadyAssigned   # relational (V2)
    return teams.add_member(team.id(), entity.id())

error AssignmentServiceError
  Entity(EntityError)        transparent
  Team(TeamError)            transparent
  EntityNotFound(EntityId)
  TeamNotFound(TeamId)
  AlreadyAssigned { entity: EntityId, team: TeamId }
```
