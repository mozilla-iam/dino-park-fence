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

Read the [DinoPark Introduction] and [Rust usage] for more information.

[people.mozilla.org]: https://people.mozilla.org
[CIS profile]: https://github.com/mozilla-iam/cis_profile-rust
[publisher]: https://auth.mozilla.com/.well-known/mozilla-iam-publisher-rules
[DinoPark Search]: https://github.com/mozilla-iam/dino-park-search
[DinoPark Tree]: https://github.com/mozilla-iam/dino-park-tree
[DinoPark Introduction]: https://github.com/mozilla-iam/dino-park/blob/master/Introduction.md
[Rust usage]: https://github.com/mozilla-iam/dino-park/blob/master/Rust.md

## Deploying

This application must be manually deployed, until we migrate our builds to
GitHub Actions.

To deploy to the development and staging clusters, run:

```
AWS_PROFILE=iam-admin aws codebuild start-build \
    --project-name dino-park-fence \
    --environment-variables-override 'name=MANUAL_DEPLOY_TRIGGER,value=branch/master'
```

To deploy to the production environment, first cut a release (or tag) in the
form:

```
<MAJOR>.<MINOR>.<PATCH>-prod
```

Then run:

```
AWS_PROFILE=iam-admin aws codebuild start-build \
    --project-name dino-park-fence \
    --environment-variables-override 'name=MANUAL_DEPLOY_TRIGGER,value=tag/<MAJOR>.<MINOR>.<PATCH>-prod'
```
