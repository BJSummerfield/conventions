# E — Edge: the API layer

The edge is where untrusted input becomes domain types and domain results
become wire shapes. Both crossings are explicit conversions, and the error
the client sees is a deliberate public type, not a leaked internal one.

## Rules

### E1 — The edge mirrors the slices
Per resource: `requests` (in), `responses` (out + errors), `routes`
(handlers). Serialization derives exist only here.

**Check:** serialization annotations appear only under the edge directory.

### E2 — One request type per action, every field optional
Every body field is optional, unknown fields are rejected. Missing input
surfaces as a field-level validation error, never a deserializer rejection.

**Check:** request types have only optional fields and reject unknown keys.

### E3 — The inbound crossing parses everything before failing
`Request -> DomainType` conversion lives in the edge slice. It parses
**every field before returning** — collecting all failures, not
short-circuiting — then finishes through the domain constructor, which
remains the single source of validation truth.

**Check:** a payload with three bad fields yields three field errors. No
validation rule is restated at the edge.

### E4 — Presence is the edge's concern; validity is the domain's
A `required(field, value, parse)` helper reports absence; the domain has no
concept of a missing field.

**Check:** no domain error variant means "field missing".

### E5 — Cross-field failures land where a form would highlight
Fields belonging to another slice are parsed separately into that slice's
types, with failures in their own error slot.

### E6 — PUT takes the full payload
No read-then-merge in handlers; the conversion is total.

### E7 — One response type per action, owning its status
Each action's response type owns its status code (create → 201, delete →
204 no body, read → 200). Identical JSON today is not a reason to merge
types — per-action wrappers around one slice-owned body, so a view can
diverge later as a local change.

**Check:** no domain type implements the framework's response interface.

### E8 — The outbound crossing reads through getters
`DomainType -> ResponseType` conversion; domain types are never serialized
directly.

### E9 — Embedded shapes are owned by the slice that renders them
No shared DTOs (S6).

### E10 — The validation error is the render context
Each slice has one validation type holding the **submitted values** plus a
**message slot per field** (empty = valid). It is the conversion's error type
and the 400 body.

### E11 — One error enum per action, structured variants only
`Validation(ValidationErrors)`, `NotFound { resource, id }`,
`Conflict { message }`, `Internal(source: ServiceError)`. No bare-string
variants outside `Internal`'s source chain.

### E12 — Service errors route per action
A conversion from the service error per action error. The same `NotFound`
may be a 404 on update but a field error on create (a bad reference in the
payload).

### E13 — Service-stage validation re-attaches submitted values
So the 400 body is identical whether the failure happened at conversion or
in the service.

### E14 — Variants map to statuses; 500 withholds detail
400 / 404 / 409 / 500. **500 bodies withhold all detail**; the source chain
is for the log. 404 bodies carry `{error, resource, id}`.

**Check:** a test asserts the exact 500 body contains no internal text.

### E15 — Handlers are thin
Extract → convert (checking all conversions before failing, so errors
combine) → one service call → wrap. No business logic, no field validation,
no inline wire shapes.

**Check:** each handler body is a handful of lines with exactly one service
call.

## Pseudocode

```
request CreateEntityRequest            # E2
  name: Option<string>
  team_id: Option<string>
  reject unknown fields

validation EntityValidationErrors      # E10
  name_value: string,    name_error: Option<string>
  team_id_value: string, team_id_error: Option<string>

convert CreateEntityRequest -> CreateEntity | EntityValidationErrors    # E3
  name    = required("name", req.name, EntityName.new)     # E4
  team_id = required("team_id", req.team_id, TeamId.new)   # E5: other slice's type
  if any failed: return errors with submitted values
  return CreateEntity.new(name, team_id)                   # cross-field rules (N9)

handler create_entity(body)                                 # E15
  command = convert(body) or return CreateEntityError.Validation(errors)
  entity  = service.create(command) or return CreateEntityError.from(service_error)
  return EntityCreatedResponse.from(entity)                 # 201 (E7)

error CreateEntityError                                     # E11
  Validation(EntityValidationErrors)      -> 400
  Conflict { message }                    -> 409
  Internal(source: EntityServiceError)    -> 500, body {"error":"internal error"}   # E14
```

Rust: [languages/rust/edge.md](../languages/rust/edge.md).
