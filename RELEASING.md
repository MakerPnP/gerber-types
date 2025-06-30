# Releasing

Set variables:

    $ export VERSION=X.Y.Z
    $ export GPG_KEY=6A72E5F0D50477236218D9D353C681785FD4B8F9

Update version numbers:

    $ vim -p Cargo.toml

Update changelog:

    $ vim CHANGELOG.md

Commit & tag:

    $ git commit -S${GPG_KEY} -m "Release v${VERSION}"
    $ git tag -s -u ${GPG_KEY} v${VERSION} -m "Version ${VERSION}"

Publish:

    $ cargo publish
    $ git push && git push --tags
