# Publishing a new Release

When a new git tag is pushed, the CI automatically creates a GitHub release, builds a new container and pushes it to the container registry.

## Checklist

Some manual steps are still necessary before handing off to the CI:

- [ ] Change the "Unreleased" section header in CHANGELOG.md to the new version number
- [ ] Update the version number in Cargo.toml
- [ ] Create a new tag using `git tag -a`
