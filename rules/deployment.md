# Deployment

## Server OS

- Use Debian for a VM, a container base image and a build image. Do not use Ubuntu.
- Use the same Debian release for the build image and the runtime image or VM. Then the binary and the system libraries agree.
- On GCE, use the image project `debian-cloud`.

## Public access

Before you design a deploy, read how the other repos in `conf/projects.json` deploy. Use the same design.

## gcloud commands

- You can run a `gcloud` command that only reads data.
- Do not run a `gcloud` command that creates, changes or deletes a resource. Only the user runs this command.
- For a command that the user runs, give the full command. Then tell the user to run it with the `!` prefix, so that the output comes into the conversation.
- A VM accepts SSH only through IAP. In each `gcloud compute ssh` or `gcloud compute scp` command, add `--tunnel-through-iap`.
