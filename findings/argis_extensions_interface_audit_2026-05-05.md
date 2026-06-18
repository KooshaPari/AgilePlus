# argis-extensions Interface Audit — 2026-05-05

## Summary

`go vet ./...` reveals **5 categories** of interface mismatches in argis-extensions.

## Issues

### 1. `api/graphql/resolvers/resolver.go` — QueryResolver mismatch (HIGH)
```
*queryResolver does not implement gen.QueryResolver
  have: Models(context.Context, *int, *string, *model.ModelFilter) (*model.ModelConnection, error)
  want: Models(context.Context) ([]*model.Model, error)
```
Schema expects no args; resolver provides 4 args. Likely: schema was simplified, resolver wasn't updated.

### 2. `api/graphql/resolvers/resolver.go` — MutationResolver mismatch (HIGH)
```
*mutationResolver does not implement gen.MutationResolver
  missing method: UpdateModel
```
Resolver is missing `UpdateModel` method that the generated GraphQL schema expects.

### 3. `api/graphql/resolvers/resolver.go` — SubscriptionResolver mismatch (HIGH)
```
*subscriptionResolver does not implement gen.SubscriptionResolver
  missing method: RoutingUpdates
```
Resolver is missing `RoutingUpdates` subscription method.

### 4. `api/graphql/resolvers/mutation.go:46` — Undefined CreateModel (MEDIUM)
```
r.models.CreateModel undefined (type ModelStore has no field or method CreateModel)
```
ModelStore is missing a `CreateModel` method. Either the schema added it or the code removed it.

### 5. `bifrost/core/schemas/schemas.go:485` — Syntax error (CRITICAL, blocks build)
```
syntax error: unexpected / in struct type; possibly missing semicolon or newline
```
File has a Go syntax error at line 485 — repo likely doesn't compile.

### 6. `db/db_test.go:304` — Undefined MinConns (LOW, test only)
```
stats.MinConns undefined (type *pgxpool.Stat has no field or method MinConns)
```
pgxpool.Stat doesn't have MinConns in this version. Test needs updating.

## Multi-Module Structure

argis-extensions has **nested Go modules**:
- Main module: `github.com/kooshapari/bifrost-extensions`
- Bifrost module: `github.com/kooshapari/bifrost` (at `./bifrost/core/`)
- SLM module: `./slm-server/`

Schema drift involves generated GraphQL code (`api/graphql/gen/`) and the resolvers that implement it.

## Recommended Action

Priority order:
1. **ARCHITECTURAL DECISION NEEDED** — the GraphQL schema (generated) and resolvers have drifted. Need to determine: regenerate schema from source, or update resolvers to match generated schema.
2. The `models` field in resolvers (`ModelStore`) needs `CreateModel` method
3. `subscriptionResolver` needs `RoutingUpdates` with updated signature: `HealthUpdates(ctx, []string)` not `HealthUpdates(ctx)`
4. `mutationResolver` needs `UpdateModel` method
5. `account_test.go` mock type mismatch — `*MockAccount` can't satisfy `*EnhancedAccount`
6. `db_test.go` MinConns — pgxpool version mismatch

**Do NOT attempt partial fixes** — the GraphQL schema and resolver signatures must be synchronized.
