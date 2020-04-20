# DinoPark Fence (protecting Dinos since 2019)
[![Build Status](https://travis-ci.org/mozilla-iam/dino-park-fence.svg?branch=master)](https://travis-ci.org/mozilla-iam/dino-park-fence)

DinoPark Fence is the main API for profile interaction for [people.mozilla.org]

# Provided Endpoints

- `/api/v4/graphql`
  - the main API for individual profile interactions
  - graphql schema used from [CIS profile]
  - retrieve data according to scope
  - modify fields owned by the _mozilliansorg_ [publisher]
- `/api/v4/search/simple/?q=<query>&w=<all|staff|contributors>`
    - search for profiles via [DinoPark Search] according to scope
- `/api/v4/orgchart/`
    - forward requests to the orgchart service
      [DinoPark Tree] (requires `staff` scope)
- `/_/login` and `/_/logout`
    - session manager (un)setting the `kli` (keep logged in) cookie and
      redirecting to our OIDC proxy
- `/metrics`
    - preliminary prometheus endpoint

# General Repository Structure

All DinoPark micro service repositories share a common structure.

## Kubernetes (k8s)

All kubernetes related configuration files are located in the `k8s` directory.
They provide a basic helm chart for the service and potential dependencies.

```ini
k8s
├── Chart.yaml
├── templates
│   ├── 00-namespace.yaml
│   ├── deployment.yaml
│   └── service.yaml
├── values # environment specific overwrites / variables
│   ├── dev.yaml
│   ├── prod.yaml
│   └── test.yaml
└── values.yaml # common variables
```

To render the chart for `dev` use:
```
helm template -f k8s/values.yaml -f k8s/values/dev.yaml k8s/
```

## Docker

The docker image produced by a repository is controlled via the `Dockerfile` or
`Dockerfile.local`. The same image is only build once and used in all environments.

## Terraform

Infrastructure is managed via terraform. All related files are located in the `terraform`
directory of the repository.

```ini
terraform
├── codebuild # codebuild and permissions for releasing
├── dev
├── prod
└── test
```

## Automation

We use [myke] (actually the [rust version of myke]). For basic automation instead of
make or shell scripts. It utilizes golang templating like helm which allows us to only
use one templating language. Look into the `myke.yaml` file to see how it works.

```bash
$ myke

 PROJECT         | TAGS | TASKS
-----------------+------+-------------------------------
 dino-park-fence |      | compile-release, deploy,
                 |      | docker, docker-local,
                 |      | package, package-local,
                 |      | push-image, run-dev

```

## CI Pipeline

### Building

DinoPark services are built and deployed via [AWS Codebuild]. See `buildspec.yaml`.

### Deploying

Every push to master will result in a build and deployment to the dev environment.
Releasing to test and prod can be done via tags. Any tag with a `-test` suffix will be
deployed to the test environment. Any tag with a `-prod` suffix will be deployed to
production.

**Note: only tag a commit that actually triggered a build**


# Anatomy of a DinoPark Rust service

All DinoPark Rust services use [actix] as their web framework. In general the
endpoints are separated into small apps:

```ini
src
├── endpoint1
│   ├── app.rs
│   ├── mod.rs
│   └── …
├── endpoint2
│   ├── app.rs
│   ├── mod.rs
│   └── …
├── healthz.rs # basic health check endpoint for k8s
├── main.rs
└── settings.rs
```

## Scopes

Permissions are and authentication happens via [DinoPark Gate]. It decodes and verifies
the id_token and translates it into _user\_id_, _scope_, _groups_scope_ and _AAL_. Routes
may be guarded to on a minimal requirement on either of those claims by [DinoPark Guard]
via a simple annotation like: `#[guard(Staff, Creator, Medium)]` which would only allow
access to this route if the logged in user is a staff member, is an allowed access group creator and is logged in via a MFA'd login method.

## CIS Integration

All services use a common [CIS client] to interact with the [CIS] APIs. Signing fields
is also supported given the correct signing keys.

## Code Style and Rules

All Rust code must pass at least:
```
cargo fmt --all -- --check
cargo clippy -- -D warnings
cargo test --all
```


[people.mozilla.org]: https://people.mozilla.org
[myke]: https://github.com/omio-labs/myke
[rust version of myke]: https://github.com/fiji-flo/myke
[AWS Codebuild]: https://aws.amazon.com/codebuild/
[actix]: https://actix.rs/
[DinoPark Gate]: https://github.com/mozilla-iam/dino-park-gate
[DinoPark Guard]: https://github.com/mozilla-iam/dino-park-guard
[CIS client]: https://github.com/mozilla-iam/cis_client-rust
[CIS]: https://github.com/mozilla-iam/cis
[CIS profile]: https://github.com/mozilla-iam/cis_profile-rust
[publisher]: https://auth.mozilla.com/.well-known/mozilla-iam-publisher-rules
[DinoPark Search]: https://github.com/mozilla-iam/dino-park-search
[DinoPark Tree]: https://github.com/mozilla-iam/dino-park-tree
