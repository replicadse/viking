# viking

![](viking.png)

[![dependency status](https://deps.rs/repo/github/replicadse/viking/status.svg)](https://deps.rs/repo/github/replicadse/viking)\
`viking` is an alternative API load testing tool. We're raiding in style.

## Project state

`viking` is unstable but really cool.

## Abstract

`viking` is an opinionated API load testing tool. A few of the key characteristics are:

- Extensive use of multithreading
- Per-thread blocking requests
- Ability to record requests and responses
- YAML based configuration
- Configurable live reporting
- Configurable error behavior
- Smart request parameters (static / increment)

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
version: "0.2"

campaigns:
  main:
    phases:
      - target:
          env: "API_URI"
        threads: 32
        ends:
          requests: 500
          #time: !s 60
        timeout:
          s: 2000
        report:
          interval:
            s: 1
        spec:
          get:
            header:
              x-api-key:
                - env: "API_KEY"
            query:
              page:
                - increment:
                    start: 0
                    step: 1
              per_page:
                - static: 4000
              from:
                - static: 1694901600
              to:
                - static: 1694905200
        behaviors:
          ok:
            - match: ^(200)$
              mark: success
            - match: .*
              mark: error
          error:
            backoff:
              s: 1

```
