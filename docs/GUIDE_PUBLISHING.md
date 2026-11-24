# Publishing the User Guide

The guide's source is `docs/guide` (mdBook; rules in `docs/guide/RULES.md`).
`.github/workflows/guide.yml` builds it and copies it into the website repo
(`operandi3d/website`, folder `public/guide/`). Cloudflare serves that
folder at operandi3d.org a minute after each push there.

| When | Goes to |
|---|---|
| A push to main that changes the guide or its scripts | operandi3d.org/guide/test/ |
| A release tag `vX.Y.Z` | operandi3d.org/guide/X.Y.Z/, and /guide/latest/ when it is the newest release |
| A pre-release tag (`v0.2.0-beta.1`) | its own folder, never latest |
| Run by hand (Actions > Publish guide > Run workflow) | as for the branch or tag picked |

operandi3d.org/guide/ goes to latest/ (test/ until the first release has a
guide). Help > User Guide in the app opens /guide/<its version>/, or
/guide/test/ in a test build. Every help id in a page header gets a
redirect at /guide/<version>/go/<help-id>/; an id no page claims lands on
the guide's search.

Build it locally with `scripts/guide-build.sh` (needs `brew install mdbook`).

## One-time setup: the WEBSITE_DEPLOY_TOKEN secret

The workflow pushes to the website repo with a token that can write to
that repo and nothing else. Until the secret exists, the workflow fails at
"Check out the website" and publishes nothing.

1. On github.com, click your picture (top right) > **Settings** >
   **Developer settings** (bottom of the left list) > **Personal access
   tokens** > **Fine-grained tokens** > **Generate new token**.
2. **Token name**: `Operandi guide publish`.
3. **Resource owner**: `operandi3d`.
4. **Expiration**: 1 year (GitHub's longest unless the organization allows
   tokens that never expire). Put a reminder in your calendar to renew it.
5. **Repository access**: **Only select repositories** > pick
   `operandi3d/website`.
6. **Permissions** > **Repository permissions** > **Contents** > **Read and
   write**. (GitHub adds Metadata: Read-only by itself.) Leave everything
   else at No access.
7. Click **Generate token** and copy it (GitHub shows it only once).
8. If GitHub says the organization must approve it: operandi3d's
   **Settings** > **Personal access tokens** > **Pending requests** >
   approve it. If fine-grained tokens are turned off for the organization,
   first allow them there (**Settings** > **Personal access tokens** >
   **Settings** > Allow access via fine-grained personal access tokens).
9. Go to the `operandi3d/operandi` repo > **Settings** > **Secrets and
   variables** > **Actions** > **New repository secret**. Name:
   `WEBSITE_DEPLOY_TOKEN`. Secret: paste the token. Click **Add secret**.

To renew: make a new token the same way and paste it over the old secret
(the secret's **Update** button).

## Notes

- The job runs on GitHub's Linux machine (about a minute, a few seconds
  of the account's Actions minutes), only for pushes and runs by the
  maintainer.
- It commits to the website repo as "Operandi guide bot", one commit per
  publish, message `guide/<folder>: from operandi <commit>`.
- A guide folder is replaced whole each time, so pages removed from the
  guide disappear from the site too. Old release folders stay.
