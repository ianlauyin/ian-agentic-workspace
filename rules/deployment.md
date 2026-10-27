# Deployment

## Server OS

- Use Debian for a VM, a container base image and a build image. Do not use Ubuntu.
- Use the same Debian release for the build image and the runtime image or VM. Then the binary and the system libraries agree.
- On GCE, use the image project `debian-cloud`.
