# O — Observability

Logging is a capability like any other, and it happens in exactly one place
per failure: the edge that drove the work.

## Rules

### O1 — Logging is a port
The domain defines a `Logger` interface in domain terms; `TracingLogger` (or
equivalent) is the adapter. The vendor logging library is named only in that
adapter and in bootstrap.

### O2 — Below the edge, nothing logs
Domain code, services, and adapters return errors (X7). The edge decides what
is worth recording.

**Check:** no log/print calls in domain, service, or adapter files (the
logging adapter itself excepted).

### O3 — One sink, installed once at bootstrap
Filter level comes from configuration (C).

### O4 — One span per request, at the edge
Method, path, status, latency. The only place request logging happens. In a
multi-service system each service logs exactly its own boundary; nothing is
double-reported.

### O5 — Each 5xx is logged exactly once
At the edge, with its full source chain (X5). Nothing else logs it on the
way up.

### O6 — Every driving adapter is an edge
A cron tick runner, a queue consumer, a CLI command each drive the same
services and each logs its own failures once — a scheduled sync's runner
holds the logger exactly like the HTTP failure middleware does. The service
never logs and never knows what drove it.

## Pseudocode

```
edge middleware on_failure(error: ActionError)
  if error is Internal(source):
    logger.error(source.to_string())       # full chain, one call (X5, O5)
  return error.into_response()             # redacted (E14)

edge cron_runner tick()
  match sync_service.run():
    ok(report) -> logger.info(report.summary())
    err(e)     -> logger.error(e.to_string())   # O6
```
