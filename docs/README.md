# viking

[![dependency status](https://deps.rs/repo/github/replicadse/viking/status.svg)](https://deps.rs/repo/github/replicadse/viking)\
`viking` is an alternative API load testing tool. We're raiding in style.

## Project state

`viking` is unstable.

## Installing

```bash
cargo install viking
```

## Example configuration

```bash
# This command renders an example configuration to STDOUT.
viking init
```

```yaml
version: "0.1"

campaigns:
  main:
    phases:
      - target: !env "API_URI"
        threads: 32
        ends:
          requests: 2000
          time: !s 60
        timeout: !s 2000
        spec: !get
          header:
            x-api-key:
              - !env "API_KEY"
          query:
            page:
              - !static 0
            per_page:
              - !static 1000
            from:
              - !static 1262304000
            to:
              - !static 1262307600
        behaviours:
          ok:
            - match: ^(200)$
              mark: !success
            - match: .*
              mark: !error
          error:
            backoff: !s 1000

```
