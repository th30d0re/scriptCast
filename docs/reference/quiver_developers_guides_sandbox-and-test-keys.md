---
title: "Sandbox and test keys"
description: "How QuiverAI's test-mode keys work: no model dispatch, no cost, deterministic output, and the markers that identify it."
icon: "flask-conical"
---

Every QuiverAI API key is created for one environment, chosen when the key is created and fixed
for the key's life: **production** (`sk_live_...`) or **test** (`sk_test_...`). This page covers
the test environment — the sandbox.

## What a test key does

A verified test key **never reaches a model.** `api.quiver.ai` routes the request to an in-process
mock instead of dispatching inference, and returns a response built from the exact same published
schema a production call would return. There is no separate sandbox host: use
`https://api.quiver.ai`. Every Production endpoint has a sandbox fixture.

Because no inference happens:

- **Nothing is billed.** Sandbox requests carry no credit cost.
- **The response is deterministic.** The same request body from the same key produces byte-identical
  output every time, including across process restarts — useful for asserting on a response in your
  own test suite instead of matching loosely.
- **Ordinary rate limits still apply.** A test key is not exempt from rate limiting — request
  flooding is still an abuse vector even against a mock — but sandbox capacity is resolved from its
  own budget, independent of your organization's paid tier. Evaluating the API does not require
  buying capacity first, and sandbox requests do not consume Production capacity buckets.

## The sandbox serves the same models production would

Model visibility and admission mirror production for your organization. `GET /v1/models` on a test
key lists exactly what it would list for a live key on the same organization — same catalog, same
preview rollouts, Arrow 2 included — and every listed model is callable on the SVG routes the
sandbox serves. A model your organization is not entitled to returns `404 model_not_found` for both
keys, so a sandbox integration test proves your production model access rather than a broader
sandbox catalog.

Token-priced responses carry the same `usage` block production returns:

```json
{
  "usage": { "input_tokens": 120, "output_tokens": 220, "total_tokens": 340 }
}
```

Those counts are mock values, fixed by the same determinism as the rest of the response. Nothing is
billed and nothing is counted against your token capacity — parse the block and assert on its shape,
not on what it would have cost.

## Identifying a sandbox response

Every response served to a test key carries a response header:

```http
x-quiver-environment: test
```

Check for this header in any code path that treats a response as billable or as real output — for
example before caching a generation, queuing it for review, or writing it to a datastore your
production pipeline also reads from.

## The SVG markers

Every SVG a sandbox response returns is marked twice in the artwork itself — once for your code,
once for your eyes.

The machine-readable half is an attribute on the root `<svg>` element:

```xml
<svg data-quiverai-sandbox="true" viewBox="0 0 24 24">...</svg>
```

The visible half is a banner drawn across the top of the artwork, reading **SANDBOX RESULT**. It is
the document's last child, a nested `<svg>` carrying its own attribute:

```xml
<svg data-quiverai-sandbox-label="true" ...>...SANDBOX RESULT...</svg>
```

Sized in percentages, it scales with whatever artwork it sits on, so a rendered sandbox SVG says
what it is without anybody having to read its source. The only SVG without the banner is a
self-closing root, which has no closing tag to put it before. It still carries the root attribute.

This is deliberate and unmistakable: a mock SVG should never be reviewable, shippable, or cacheable
as a real generation. If your pipeline stores or renders generated SVGs, check for the attribute
before treating one as production output — the same rule as the response header, expressed in the
artifact itself so it survives being copied out of the response body.

## Triggering specific outcomes

The sandbox recognizes a marker you can put in a request field to deliberately exercise
edge-case handling before production exercises it for you:

| Marker               | Where                                  | Effect                                                                 |
| -------------------- | -------------------------------------- | ---------------------------------------------------------------------- |
| `[mock:slow-stream]` | in the `prompt` of a streaming request | slows the stream down, useful for testing client-side timeout handling |

This marker is inert in production — sending it to a production key does not do anything
special, since production never inspects the prompt for a sandbox marker.

## `/v1/responses` in the sandbox

A test key calls `/v1/responses` the same way a Production key does, and gets the same
`ResponseResource` shape back — `id`, `status`, `output`, `usage` — built from the published
schema rather than from a model.

Without `tools`, the answer is an assistant message whose text quotes the SVG in a fenced
` ```svg ` block. Declare a `write_file` function tool and the sandbox answers with
`function_call` output items instead, whose `arguments` carry the `path` and `content` your
integration would write.

Set `"stream": true` and the sandbox streams the same event sequence, from `response.created`
through `response.completed`, terminated by `data: [DONE]`.

Every SVG in either shape carries both sandbox markers described above, inside the fenced block or
inside the serialized tool-call arguments.

To run the Open Responses conformance suite against a key, see
[Run the Open Responses conformance suite](/developers/guides/open-responses-conformance).

## Next steps

- Create a test key from [API Keys](https://platform.quiver.ai/api-keys) — the environment
  choice is made once, at creation, and cannot be changed later.
- Use [Text to SVG](/developers/models/text-to-svg) or [Image to SVG](/developers/models/image-to-svg)
  for native request examples. The [Quickstart](/developers/quickstart) uses the Responses API,
  which a test key can call too.
- See [pricing and plans](/developers/pricing) — sandbox usage is excluded from spend entirely.
