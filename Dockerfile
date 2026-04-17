FROM gcr.io/distroless/static-debian12:nonroot

ARG TARGETARCH

COPY portainer-stacks-${TARGETARCH} /usr/local/bin/portainer-stacks

ENTRYPOINT ["/usr/local/bin/portainer-stacks"]
