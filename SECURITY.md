# Security Policy

## Scope

This is a static educational website plus small, dependency-light example
programs. The most relevant "security" concerns are:

- Example code that could be copied into real systems.
- Supply-chain integrity of the site's build dependencies.

We take care that example code does not model insecure patterns without clearly
labeling them as anti-patterns.

## Reporting a vulnerability

If you find a security issue — in the build tooling, a dependency, or an example
that teaches an unsafe pattern as if it were safe — please email
**ss.system@kaeso.co** with:

- A description of the issue and where it appears.
- Steps to reproduce, if applicable.

Please do **not** open a public issue for security reports. We aim to acknowledge
reports within 5 business days.

## Supported versions

The `main` branch is the only supported version; fixes land there.
