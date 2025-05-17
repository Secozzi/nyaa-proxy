# Nyaa proxy

Simple web-server to run a private nyaa proxy with link rewrites. A script that will copy the rss link upon click is also injected.

## Environment variables

- `PORT` port used, defaults to 3000.
- `NYAA_URL` url of nyaa, defaults to `https://nyaa.si`.
- `PROXY_URL` url of the proxy, defaults to `https://nyaa.si`, used for link rewrites.

## Credits

- Dockerfile is based on https://github.com/anlumo/rust-service-template with some tweaks to get reqwest working with https.
- Github workflow from https://medium.com/@jaredhatfield/publishing-semantic-versioned-docker-images-to-github-packages-using-github-actions-ebe88fa74522
