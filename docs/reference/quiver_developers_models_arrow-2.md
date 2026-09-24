---
title: "Arrow 2"
description: "A balance of generation quality and speed for SVG integrations."
sidebar:
  hidden: true
---

![Arrow 2 cover](/blume-assets/content/developers/models/images/arrow-2.png)

Arrow 2 balances SVG generation quality and speed. Its API model ID is `arrow-2`, not
`arrow-2.0`.

## Availability

`arrow-2` is generally available. List it with
[`GET /v1/models`](/api-reference/models/list-models) using either a Production or a Test key.
Your key must also authorize the model and operation.

The public API supports SVG generation, vectorization, editing, and animation when enabled for
your key. Use Responses only when the returned `supported_operations` includes `open_responses`.
Check the matching operation before calling a native SVG endpoint.

## Model details

| Property         | Value            |
| ---------------- | ---------------- |
| API model ID     | `arrow-2`        |
| Input modalities | Text, image, SVG |
| Output modality  | SVG              |
| Context window   | 131,072 tokens   |
| Maximum output   | 65,536 tokens    |
| Billing kind     | `token_usage`    |

Supported request fields vary by endpoint.

## Pricing

| Input | Cached input | Cache write | Output |
| ----- | ------------ | ----------- | ------ |
| $4.00 | $0.40        | $5.00       | $20.00 |

All prices are **USD per 1 million tokens**. Arrow 2 has lower token rates than Arrow 2 Telos;
Telos rates are 50% higher in every category. Charges depend on measured token usage, not a fixed
price per SVG. Read current rates from `billing.rates` and see [Pricing](/developers/pricing) for
cache accounting and a worked comparison.

## Start building

Follow the [Quickstart](/developers/quickstart) to generate and save an SVG. For an existing native
integration, see [Migrate to Responses](/developers/guides/migrate-to-responses).

[Compare all models](/developers/models).
