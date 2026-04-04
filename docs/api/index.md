# API

The AgilePlus API provides programmatic access to all project management features.

## Base URL

```
https://api.agileplus.local/v1
```

## Authentication

All requests require a Bearer token:

```bash
curl -H "Authorization: Bearer <token>" https://api.agileplus.local/v1/projects
```

## Endpoints

### Projects

- `GET /projects` - List all projects
- `POST /projects` - Create a new project
- `GET /projects/{id}` - Get project details
- `PUT /projects/{id}` - Update project
- `DELETE /projects/{id}` - Delete project

### Features

- `GET /features` - List all features
- `GET /features?project={id}` - List features by project
- `POST /features` - Create a new feature
- `GET /features/{id}` - Get feature details
- `PUT /features/{id}` - Update feature
- `DELETE /features/{id}` - Delete feature

### Work Packages

- `GET /work-packages` - List all work packages
- `GET /work-packages?feature={id}` - List work packages by feature
- `POST /work-packages` - Create a new work package
- `GET /work-packages/{id}` - Get work package details
- `PUT /work-packages/{id}` - Update work package
- `PUT /work-packages/{id}/status` - Update work package status
- `DELETE /work-packages/{id}` - Delete work package

## OpenAPI Specification

The complete OpenAPI spec is available at `/openapi.yaml`.
