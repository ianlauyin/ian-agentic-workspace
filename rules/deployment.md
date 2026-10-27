# Deployment

## Server OS

- Use Debian for a VM, a container base image and a build image. Do not use Ubuntu.
- Use the same Debian release for the build image and the runtime image or VM. Then the binary and the system libraries agree.
- On GCE, use the image project `debian-cloud`.

## gcloud commands

- Do not run a `gcloud` command yourself. Only the user runs a `gcloud` command. This rule applies also to a command that only reads data.
- Give the full command to the user. Then tell the user to run it with the `!` prefix, so that the output comes into the conversation.
