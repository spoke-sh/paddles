# Paddles Public Docs

This directory contains the public MDX documentation site for Paddles.

## Local Workflow

Use the repo-supported commands from the repository root:

```bash
just frontend-install
npm --workspace @paddles/docs run start
npm --workspace @paddles/docs run build
```

These commands use the repository's shared frontend workspace so the docs
workflow stays reproducible in this checkout.

For production publication, the repository-owned
[`publish-docs.yml`](../../.github/workflows/publish-docs.yml) workflow is the
preferred lane. It publishes the stable Paddles site plus the `main` preview
into the shared `spoke-previews` bucket through the infra-managed OIDC role.
The checked-in [`publish-docs.sh`](../../scripts/publish-docs.sh) script is the
local repair and CI execution surface for that contract. Published docs objects
use `Cache-Control: no-cache` so browsers revalidate with `ETag` and
`Last-Modified`.

## Build Inputs

The site reads these optional environment variables at build time:

- `DOCS_SITE_URL`
- `DOCS_BASE_URL`

If they are not set, the site defaults to `https://paddles.spoke.sh` and `/`.

## Deployment Inputs

The shared production docs lane is:

- stable docs at `https://www.spoke.sh/paddles/docs`
- preview docs at `https://www.spoke.sh/previews/paddles/<branch>/docs`

The publish workflow runs in the repository's `prod` GitHub environment. It
accepts the same publication inputs the sibling repos use:

- `AWS_ROLE_TO_ASSUME` (optional when the default `spoke-paddles-docs-publisher` role ARN is correct)
- `DOCS_PREVIEW_BUCKET` (optional; defaults to `spoke-previews`)

The publish script also accepts:

- `DOCS_APP_NAME`
- `DOCS_SITE_URL`
- `DOCS_BRANCH`
- `DOCS_PUBLISH_STABLE`
- `DOCS_SKIP_SYNC`
