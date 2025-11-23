# Publishing a new Release

When a new git tag is pushed, the CI automatically creates a GitHub release, builds a new container and pushes it to the container registry.

## Checklist

Some manual steps are still necessary before handing off to the CI:

- [ ] Change the "Unreleased" section header in CHANGELOG.md to the new version number
- [ ] Update the version number in Cargo.toml
- [ ] Commit using the conventional message "Release v<version>"
- [ ] Create a new tag using `git tag -a v<version>`
- [ ] Push the commit and tag: `git push --tags`
- [ ] Wait for the CI to finish, then publish the draft release using the GitHub UI. Once this is done, the release is immutable!

## Testing changes to the release CI

You can skip side-effects such as publishing the containers by including `test-release` in your tag name, e.g. `v0.1.0test-release`.
The `test-release` string will be removed automatically when extracting the version number from the release.
This allows you to run the release CI in a pull request without actually publishing a release.
The github release will still be created as a draft, so you'll have to manually delete it afterwards.

If you make changes, you'll need to move the test tag to the new commit by first deleting it on the remote: `git push origin :v0.1.0test-release` and then pushing it again.
